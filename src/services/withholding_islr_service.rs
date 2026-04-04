//! Servicio de retenciones de ISLR: persistencia real con SeaORM.

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, Statement, TransactionTrait,
};
use uuid::Uuid;

use crate::domain::numbering::rif::Rif;
use crate::domain::withholding::islr_wh::calculate_islr_withholding_with_subtract;
use crate::dto::PaginatedResponse;
use crate::dto::withholding_islr_dto::{
    CreateIslrWithholdingRequest, IslrWithholdingFilters, IslrWithholdingListResponse,
    IslrWithholdingResponse,
};
use crate::entities::{invoices, tax_withholdings_islr};
use crate::errors::AppError;
use crate::services::audit_service::{self, AuditAction, AuditEntity};

/// Valida que el periodo tenga formato YYYY-MM (ej: "2026-03").
fn is_valid_islr_period(period: &str) -> bool {
    if period.len() != 7 {
        return false;
    }
    let parts: Vec<&str> = period.split('-').collect();
    if parts.len() != 2 {
        return false;
    }
    let year_part = parts[0];
    let month_part = parts[1];
    if year_part.len() != 4 || month_part.len() != 2 {
        return false;
    }
    if !year_part.chars().all(|c| c.is_ascii_digit())
        || !month_part.chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    let month: u32 = match month_part.parse() {
        Ok(m) => m,
        Err(_) => return false,
    };
    (1..=12).contains(&month)
}

/// Convierte el porcentaje del DTO (ej: 5 para 5%) a fracción decimal (0.05).
fn percentage_to_rate(percentage: Decimal) -> Result<Decimal, AppError> {
    let rate = percentage / dec!(100);
    if rate < Decimal::ZERO || rate > dec!(1.00) {
        return Err(AppError::BadRequest(format!(
            "Tasa de retención ISLR inválida: {}%. Debe estar entre 0% y 100%",
            percentage
        )));
    }
    Ok(rate)
}

/// Obtiene el próximo número ARC atómicamente usando SELECT FOR UPDATE en numbering_sequences.
async fn next_arc_number_atomic<C: ConnectionTrait>(
    txn: &C,
    company_profile_id: Uuid,
) -> Result<String, AppError> {
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"SELECT id, current_value, prefix FROM numbering_sequences
               WHERE company_profile_id = $1 AND sequence_type = 'ARC' AND is_active = true
               FOR UPDATE"#,
            [company_profile_id.into()],
        ))
        .await
        .map_err(AppError::Database)?;

    let row = rows.first().ok_or_else(|| {
        AppError::BadRequest(
            "No hay secuencia de numeración activa para comprobantes ARC. Ejecute el seeder.".into(),
        )
    })?;

    let id: Uuid = row.try_get("", "id").map_err(AppError::Database)?;
    let current: i64 = row.try_get("", "current_value").map_err(AppError::Database)?;
    let prefix: Option<String> = row.try_get("", "prefix").map_err(AppError::Database)?;

    let next_val = current + 1;

    txn.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"UPDATE numbering_sequences SET current_value = $1, updated_at = NOW() WHERE id = $2"#,
        [next_val.into(), id.into()],
    ))
    .await
    .map_err(AppError::Database)?;

    let number = format!("{}{:08}", prefix.unwrap_or_else(|| "ARC-".to_string()), next_val);
    Ok(number)
}

/// Crea una nueva retención de ISLR con persistencia real.
pub async fn create_islr_withholding(
    db: &DatabaseConnection,
    dto: CreateIslrWithholdingRequest,
    user_id: Uuid,
    company_profile_id: Uuid,
) -> Result<IslrWithholdingResponse, AppError> {
    if dto.beneficiary_rif.trim().is_empty() {
        return Err(AppError::Validation(
            "El RIF del beneficiario es obligatorio".to_string(),
        ));
    }
    Rif::parse(&dto.beneficiary_rif).map_err(|e| {
        AppError::Validation(format!("RIF del beneficiario inválido: {}", e))
    })?;
    if dto.beneficiary_name.trim().is_empty() {
        return Err(AppError::Validation(
            "El nombre del beneficiario es obligatorio".to_string(),
        ));
    }
    if dto.activity_type.trim().is_empty() {
        return Err(AppError::Validation(
            "El tipo de actividad es obligatorio".to_string(),
        ));
    }
    if dto.period.trim().is_empty() {
        return Err(AppError::Validation(
            "El periodo es obligatorio".to_string(),
        ));
    }
    if !is_valid_islr_period(&dto.period) {
        return Err(AppError::Validation(
            "El periodo debe estar en formato YYYY-MM (ej: 2026-03)".to_string(),
        ));
    }

    let rate = percentage_to_rate(dto.withholding_rate)?;
    let subtract = dto.subtract_amount.unwrap_or(Decimal::ZERO);

    let result = calculate_islr_withholding_with_subtract(dto.base_amount, rate, subtract)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let txn = db.begin().await?;

    // Validate that the referenced invoice actually exists
    invoices::Entity::find_by_id(dto.invoice_id)
        .one(&txn)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Factura con ID {} no encontrada. No se puede registrar la retención.",
                dto.invoice_id
            ))
        })?;

    // Número ARC atómico — igual que el resto de documentos fiscales
    let arc_number = next_arc_number_atomic(&txn, company_profile_id).await?;

    // Resolve supplier_id dentro de la transacción
    let supplier_id =
        resolve_supplier_id(&txn, &dto.beneficiary_rif, &dto.beneficiary_name, user_id).await?;

    let id = Uuid::new_v4();
    let now = Utc::now().into();

    let withholding = tax_withholdings_islr::ActiveModel {
        id: Set(id),
        arc_number: Set(arc_number),
        invoice_id: Set(dto.invoice_id),
        supplier_id: Set(supplier_id),
        company_profile_id: Set(company_profile_id),
        withholding_date: Set(now),
        reporting_period: Set(dto.period.clone()),
        activity_code: Set(dto.activity_type.clone()),
        activity_description: Set(dto.activity_type.clone()),
        taxable_amount: Set(dto.base_amount),
        withholding_rate: Set(dto.withholding_rate),
        withheld_amount: Set(result.withheld_amount),
        status: Set("Emitido".to_string()),
        txt_file_path: Set(None),
        created_by: Set(user_id),
        created_at: Set(now),
        updated_at: Set(now),
    };

    withholding.insert(&txn).await?;
    txn.commit().await?;

    audit_service::log(
        db,
        user_id,
        AuditAction::Create,
        AuditEntity::WithholdingIslr,
        id,
        None,
    )
    .await;

    Ok(IslrWithholdingResponse {
        id,
        invoice_id: dto.invoice_id,
        beneficiary_rif: dto.beneficiary_rif,
        beneficiary_name: dto.beneficiary_name,
        base_amount: result.base_amount,
        activity_type: dto.activity_type,
        withholding_rate: dto.withholding_rate,
        subtract_amount: result.subtract_amount,
        withheld_amount: result.withheld_amount,
        net_payable: result.net_payable,
        period: dto.period,
        created_by: user_id,
        created_at: Utc::now(),
    })
}

async fn resolve_supplier_id<C: ConnectionTrait>(
    txn: &C,
    rif: &str,
    name: &str,
    user_id: Uuid,
) -> Result<Uuid, AppError> {
    use crate::entities::clients;

    let existing = clients::Entity::find()
        .filter(clients::Column::Rif.eq(rif))
        .one(txn)
        .await?;

    if let Some(c) = existing {
        return Ok(c.id);
    }

    let id = Uuid::new_v4();
    let now = Utc::now().into();
    let client = clients::ActiveModel {
        id: Set(id),
        razon_social: Set(name.to_string()),
        nombre_comercial: Set(None),
        rif: Set(Some(rif.to_string())),
        cedula: Set(None),
        domicilio_fiscal: Set(None),
        telefono: Set(None),
        email: Set(None),
        es_consumidor_final: Set(false),
        es_contribuyente_especial: Set(false),
        is_active: Set(true),
        created_by: Set(user_id),
        created_at: Set(now),
        updated_at: Set(now),
    };
    client.insert(txn).await?;
    Ok(id)
}

/// Obtiene una retención de ISLR por ID.
pub async fn get_islr_withholding(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<IslrWithholdingResponse, AppError> {
    let wh = tax_withholdings_islr::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Retención de ISLR con ID {} no encontrada", id))
        })?;

    let supplier = crate::entities::clients::Entity::find_by_id(wh.supplier_id)
        .one(db)
        .await?;
    let (beneficiary_rif, beneficiary_name) = match supplier {
        Some(s) => (s.rif.unwrap_or_default(), s.razon_social),
        None => (String::new(), "Desconocido".to_string()),
    };

    let net_payable = wh.taxable_amount - wh.withheld_amount;

    Ok(IslrWithholdingResponse {
        id: wh.id,
        invoice_id: wh.invoice_id,
        beneficiary_rif,
        beneficiary_name,
        base_amount: wh.taxable_amount,
        activity_type: wh.activity_code,
        withholding_rate: wh.withholding_rate,
        subtract_amount: Decimal::ZERO,
        withheld_amount: wh.withheld_amount,
        net_payable,
        period: wh.reporting_period,
        created_by: wh.created_by,
        created_at: wh.created_at.with_timezone(&Utc),
    })
}

/// Lista retenciones de ISLR con filtros y paginación.
pub async fn list_islr_withholdings(
    db: &DatabaseConnection,
    filters: IslrWithholdingFilters,
) -> Result<PaginatedResponse<IslrWithholdingListResponse>, AppError> {
    let page = filters.page.unwrap_or(1);
    let per_page = filters.per_page.unwrap_or(25);

    let mut query = tax_withholdings_islr::Entity::find()
        .order_by_desc(tax_withholdings_islr::Column::CreatedAt);

    if let Some(ref period) = filters.period {
        query =
            query.filter(tax_withholdings_islr::Column::ReportingPeriod.eq(period.as_str()));
    }

    let total = query.clone().count(db).await?;
    let offset = (page.saturating_sub(1)) * per_page;

    let models = query.offset(Some(offset)).limit(Some(per_page)).all(db).await?;

    let mut data = Vec::new();
    for wh in models {
        let supplier = crate::entities::clients::Entity::find_by_id(wh.supplier_id)
            .one(db)
            .await?;
        let (beneficiary_rif, beneficiary_name) = match supplier {
            Some(s) => (s.rif.unwrap_or_default(), s.razon_social),
            None => (String::new(), "Desconocido".to_string()),
        };

        data.push(IslrWithholdingListResponse {
            id: wh.id,
            invoice_id: wh.invoice_id,
            beneficiary_rif,
            beneficiary_name,
            base_amount: wh.taxable_amount,
            withheld_amount: wh.withheld_amount,
            period: wh.reporting_period,
            created_at: wh.created_at.with_timezone(&Utc),
        });
    }

    Ok(PaginatedResponse::new(data, page, per_page, total))
}

/// Obtiene todas las retenciones de ISLR para un periodo (para exportación TXT).
pub async fn get_islr_withholdings_for_period(
    db: &DatabaseConnection,
    period: &str,
) -> Result<Vec<IslrWithholdingResponse>, AppError> {
    let models = tax_withholdings_islr::Entity::find()
        .filter(tax_withholdings_islr::Column::ReportingPeriod.eq(period))
        .order_by_asc(tax_withholdings_islr::Column::CreatedAt)
        .all(db)
        .await?;

    let mut result = Vec::new();
    for wh in models {
        let supplier = crate::entities::clients::Entity::find_by_id(wh.supplier_id)
            .one(db)
            .await?;
        let (beneficiary_rif, beneficiary_name) = match supplier {
            Some(s) => (s.rif.unwrap_or_default(), s.razon_social),
            None => (String::new(), "Desconocido".to_string()),
        };
        let net_payable = wh.taxable_amount - wh.withheld_amount;

        result.push(IslrWithholdingResponse {
            id: wh.id,
            invoice_id: wh.invoice_id,
            beneficiary_rif,
            beneficiary_name,
            base_amount: wh.taxable_amount,
            activity_type: wh.activity_code,
            withholding_rate: wh.withholding_rate,
            subtract_amount: Decimal::ZERO,
            withheld_amount: wh.withheld_amount,
            net_payable,
            period: wh.reporting_period,
            created_by: wh.created_by,
            created_at: wh.created_at.with_timezone(&Utc),
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_percentage_to_rate_5() {
        let rate = percentage_to_rate(dec!(5)).unwrap();
        assert_eq!(rate, dec!(0.05));
    }

    #[test]
    fn test_percentage_to_rate_34() {
        let rate = percentage_to_rate(dec!(34)).unwrap();
        assert_eq!(rate, dec!(0.34));
    }

    #[test]
    fn test_percentage_to_rate_0() {
        let rate = percentage_to_rate(dec!(0)).unwrap();
        assert_eq!(rate, dec!(0.00));
    }

    #[test]
    fn test_percentage_to_rate_100() {
        let rate = percentage_to_rate(dec!(100)).unwrap();
        assert_eq!(rate, dec!(1.00));
    }

    #[test]
    fn test_percentage_to_rate_negative() {
        assert!(percentage_to_rate(dec!(-5)).is_err());
    }

    #[test]
    fn test_percentage_to_rate_above_100() {
        assert!(percentage_to_rate(dec!(150)).is_err());
    }

    #[test]
    fn test_valid_islr_period() {
        assert!(is_valid_islr_period("2026-03"));
        assert!(is_valid_islr_period("2026-01"));
        assert!(is_valid_islr_period("2026-12"));
    }

    #[test]
    fn test_invalid_islr_period_month_13() {
        assert!(!is_valid_islr_period("2026-13"));
    }

    #[test]
    fn test_invalid_islr_period_month_00() {
        assert!(!is_valid_islr_period("2026-00"));
    }

    #[test]
    fn test_invalid_islr_period_wrong_format() {
        assert!(!is_valid_islr_period("202603"));
        assert!(!is_valid_islr_period("2026/03"));
        assert!(!is_valid_islr_period("26-03"));
        assert!(!is_valid_islr_period("202603-01"));
    }

    #[test]
    fn test_invalid_islr_period_empty() {
        assert!(!is_valid_islr_period(""));
    }
}
