//! Servicio de retenciones de IVA: persistencia real con SeaORM.

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

use crate::domain::numbering::rif::Rif;
use crate::domain::withholding::iva_wh::{IvaWithholdingRate, calculate_iva_withholding};
use crate::dto::PaginatedResponse;
use crate::dto::withholding_iva_dto::{
    CreateIvaWithholdingRequest, IvaWithholdingFilters, IvaWithholdingListResponse,
    IvaWithholdingResponse,
};
use crate::entities::{invoices, tax_withholdings_iva};
use crate::errors::AppError;
use crate::services::audit_service::{self, AuditAction, AuditEntity};

/// Convierte el porcentaje (75 o 100) al enum de dominio.
fn parse_withholding_rate(rate: u8) -> Result<IvaWithholdingRate, AppError> {
    match rate {
        75 => Ok(IvaWithholdingRate::Standard),
        100 => Ok(IvaWithholdingRate::Full),
        _ => Err(AppError::BadRequest(format!(
            "Tasa de retención IVA inválida: {}%. Valores válidos: 75, 100",
            rate
        ))),
    }
}

/// Valida que el periodo tenga formato YYYYMM-QQ (ej: "202603-01" o "202603-02").
fn is_valid_iva_period(period: &str) -> bool {
    if period.len() != 9 {
        return false;
    }
    let parts: Vec<&str> = period.split('-').collect();
    if parts.len() != 2 {
        return false;
    }
    let year_month = parts[0];
    let fortnight = parts[1];
    if year_month.len() != 6 || fortnight.len() != 2 {
        return false;
    }
    if !year_month.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let month: u32 = match year_month[4..6].parse() {
        Ok(m) => m,
        Err(_) => return false,
    };
    if !(1..=12).contains(&month) {
        return false;
    }
    fortnight == "01" || fortnight == "02"
}

/// Parsed components of a YYYYMM-QQ period string.
struct ParsedIvaPeriod {
    /// DB `period` column value: "PRIMERA" or "SEGUNDA"
    period_label: String,
    /// DB `reporting_period` column value in "YYYY-MM" format
    reporting_period: String,
}

/// Parses "YYYYMM-QQ" into DB-compatible period_label and reporting_period.
///
/// Example: "202603-01" -> period_label="PRIMERA", reporting_period="2026-03"
fn parse_iva_period(period: &str) -> Result<ParsedIvaPeriod, AppError> {
    if !is_valid_iva_period(period) {
        return Err(AppError::Validation(
            "El periodo debe estar en formato YYYYMM-QQ (ej: 202603-01 o 202603-02)".to_string(),
        ));
    }
    let parts: Vec<&str> = period.split('-').collect();
    let year_month = parts[0]; // "202603"
    let fortnight = parts[1]; // "01" or "02"

    let year = &year_month[0..4];
    let month = &year_month[4..6];
    let reporting_period = format!("{}-{}", year, month);

    let period_label = match fortnight {
        "01" => "PRIMERA".to_string(),
        "02" => "SEGUNDA".to_string(),
        _ => unreachable!(), // is_valid_iva_period already checked this
    };

    Ok(ParsedIvaPeriod {
        period_label,
        reporting_period,
    })
}

/// Reconstructs the original "YYYYMM-QQ" format from DB values.
///
/// Example: period_label="PRIMERA", reporting_period="2026-03" -> "202603-01"
fn reconstruct_iva_period(period_label: &str, reporting_period: &str) -> String {
    let fortnight = match period_label {
        "PRIMERA" => "01",
        "SEGUNDA" => "02",
        _ => "00", // Fallback for unexpected DB values
    };
    // reporting_period is "YYYY-MM", remove the dash -> "YYYYMM"
    let yyyymm = reporting_period.replace('-', "");
    format!("{}-{}", yyyymm, fortnight)
}

/// Crea una nueva retención de IVA con persistencia real.
pub async fn create_iva_withholding(
    db: &DatabaseConnection,
    dto: CreateIvaWithholdingRequest,
    user_id: Uuid,
    company_profile_id: Uuid,
) -> Result<IvaWithholdingResponse, AppError> {
    let rate = parse_withholding_rate(dto.withholding_rate)?;

    if dto.supplier_rif.trim().is_empty() {
        return Err(AppError::Validation(
            "El RIF del proveedor es obligatorio".to_string(),
        ));
    }
    Rif::parse(&dto.supplier_rif).map_err(|e| {
        AppError::Validation(format!("RIF del proveedor inválido: {}", e))
    })?;
    if dto.supplier_name.trim().is_empty() {
        return Err(AppError::Validation(
            "El nombre del proveedor es obligatorio".to_string(),
        ));
    }
    if dto.voucher_number.trim().is_empty() {
        return Err(AppError::Validation(
            "El número de comprobante es obligatorio".to_string(),
        ));
    }
    if dto.period.trim().is_empty() {
        return Err(AppError::Validation(
            "El periodo es obligatorio".to_string(),
        ));
    }
    let parsed_period = parse_iva_period(&dto.period)?;

    let result = calculate_iva_withholding(dto.iva_amount, rate)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Validate that the referenced invoice actually exists
    invoices::Entity::find_by_id(dto.invoice_id)
        .one(db)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Factura con ID {} no encontrada. No se puede registrar la retención.",
                dto.invoice_id
            ))
        })?;

    // Resolve supplier_id (find or create client for supplier)
    let supplier_id = resolve_supplier_id(db, &dto.supplier_rif, &dto.supplier_name, user_id, company_profile_id).await?;

    let id = Uuid::new_v4();
    let now = Utc::now().into();
    let wh_percentage = Decimal::from(dto.withholding_rate);

    let withholding = tax_withholdings_iva::ActiveModel {
        id: Set(id),
        voucher_number: Set(dto.voucher_number.clone()),
        invoice_id: Set(dto.invoice_id),
        supplier_id: Set(supplier_id),
        company_profile_id: Set(company_profile_id),
        withholding_date: Set(now),
        period: Set(parsed_period.period_label),
        reporting_period: Set(parsed_period.reporting_period),
        iva_amount: Set(dto.iva_amount),
        withholding_percentage: Set(wh_percentage),
        withheld_amount: Set(result.withheld_amount),
        status: Set("Emitido".to_string()),
        xml_file_path: Set(None),
        created_by: Set(user_id),
        created_at: Set(now),
        updated_at: Set(now),
    };

    withholding.insert(db).await?;

    audit_service::log(
        db,
        user_id,
        AuditAction::Create,
        AuditEntity::WithholdingIva,
        id,
        Some(company_profile_id),
        None,
    )
    .await;

    Ok(IvaWithholdingResponse {
        id,
        invoice_id: dto.invoice_id,
        supplier_rif: dto.supplier_rif,
        supplier_name: dto.supplier_name,
        iva_amount: result.iva_amount,
        withholding_rate: dto.withholding_rate,
        withheld_amount: result.withheld_amount,
        net_payable: result.net_payable,
        period: dto.period,
        voucher_number: dto.voucher_number,
        created_by: user_id,
        created_at: Utc::now(),
    })
}

async fn resolve_supplier_id(
    db: &DatabaseConnection,
    rif: &str,
    name: &str,
    user_id: Uuid,
    company_profile_id: Uuid,
) -> Result<Uuid, AppError> {
    use crate::entities::clients;

    let existing = clients::Entity::find()
        .filter(clients::Column::Rif.eq(rif))
        .filter(clients::Column::CompanyProfileId.eq(company_profile_id))
        .one(db)
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
        company_profile_id: Set(company_profile_id),
    };
    client.insert(db).await?;
    Ok(id)
}

/// Obtiene una retención de IVA por ID.
pub async fn get_iva_withholding(
    db: &DatabaseConnection,
    id: Uuid,
    company_profile_id: Uuid,
) -> Result<IvaWithholdingResponse, AppError> {
    let wh = tax_withholdings_iva::Entity::find_by_id(id)
        .filter(tax_withholdings_iva::Column::CompanyProfileId.eq(company_profile_id))
        .one(db)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Retención de IVA con ID {} no encontrada", id))
        })?;

    let supplier = crate::entities::clients::Entity::find_by_id(wh.supplier_id)
        .one(db)
        .await?;
    let (supplier_rif, supplier_name) = match supplier {
        Some(s) => (s.rif.unwrap_or_default(), s.razon_social),
        None => (String::new(), "Desconocido".to_string()),
    };

    let net_payable = wh.iva_amount - wh.withheld_amount;

    let withholding_rate = wh.withholding_percentage.to_u8().ok_or_else(|| {
        AppError::Internal(format!(
            "No se pudo convertir el porcentaje de retención {} a u8 para la retención {}",
            wh.withholding_percentage, wh.id
        ))
    })?;

    Ok(IvaWithholdingResponse {
        id: wh.id,
        invoice_id: wh.invoice_id,
        supplier_rif,
        supplier_name,
        iva_amount: wh.iva_amount,
        withholding_rate,
        withheld_amount: wh.withheld_amount,
        net_payable,
        period: reconstruct_iva_period(&wh.period, &wh.reporting_period),
        voucher_number: wh.voucher_number,
        created_by: wh.created_by,
        created_at: wh.created_at.with_timezone(&Utc),
    })
}

/// Lista retenciones de IVA con filtros y paginación.
pub async fn list_iva_withholdings(
    db: &DatabaseConnection,
    filters: IvaWithholdingFilters,
    company_profile_id: Uuid,
) -> Result<PaginatedResponse<IvaWithholdingListResponse>, AppError> {
    let page = filters.page.unwrap_or(1);
    let per_page = filters.per_page.unwrap_or(25);

    let mut query = tax_withholdings_iva::Entity::find()
        .filter(tax_withholdings_iva::Column::CompanyProfileId.eq(company_profile_id))
        .order_by_desc(tax_withholdings_iva::Column::CreatedAt);

    if let Some(ref period) = filters.period {
        // The filter period can be YYYYMM-QQ (specific fortnight) or YYYY-MM (whole month)
        if is_valid_iva_period(period) {
            let parsed = parse_iva_period(period)?;
            query = query
                .filter(tax_withholdings_iva::Column::ReportingPeriod.eq(parsed.reporting_period))
                .filter(tax_withholdings_iva::Column::Period.eq(parsed.period_label));
        } else {
            // Assume YYYY-MM format for filtering by whole month
            query = query.filter(tax_withholdings_iva::Column::ReportingPeriod.eq(period.as_str()));
        }
    }

    let total = query.clone().count(db).await?;
    let offset = (page.saturating_sub(1)) * per_page;

    let models = query.offset(Some(offset)).limit(Some(per_page)).all(db).await?;

    let mut data = Vec::new();
    for wh in models {
        let supplier = crate::entities::clients::Entity::find_by_id(wh.supplier_id)
            .one(db)
            .await?;
        let (supplier_rif, supplier_name) = match supplier {
            Some(s) => (s.rif.unwrap_or_default(), s.razon_social),
            None => (String::new(), "Desconocido".to_string()),
        };

        data.push(IvaWithholdingListResponse {
            id: wh.id,
            invoice_id: wh.invoice_id,
            supplier_rif,
            supplier_name,
            iva_amount: wh.iva_amount,
            withheld_amount: wh.withheld_amount,
            period: reconstruct_iva_period(&wh.period, &wh.reporting_period),
            voucher_number: wh.voucher_number,
            created_at: wh.created_at.with_timezone(&Utc),
        });
    }

    Ok(PaginatedResponse::new(data, page, per_page, total))
}

/// Obtiene todas las retenciones de IVA para un periodo (para exportación XML).
///
/// Accepts either `YYYYMM-QQ` format (filters by specific fortnight) or
/// `YYYY-MM` format (filters by whole month).
pub async fn get_iva_withholdings_for_period(
    db: &DatabaseConnection,
    period: &str,
    company_profile_id: Uuid,
) -> Result<Vec<IvaWithholdingResponse>, AppError> {
    let mut query = tax_withholdings_iva::Entity::find()
        .filter(tax_withholdings_iva::Column::CompanyProfileId.eq(company_profile_id));

    if is_valid_iva_period(period) {
        let parsed = parse_iva_period(period)?;
        query = query
            .filter(tax_withholdings_iva::Column::ReportingPeriod.eq(parsed.reporting_period))
            .filter(tax_withholdings_iva::Column::Period.eq(parsed.period_label));
    } else {
        // Assume YYYY-MM format
        query = query.filter(tax_withholdings_iva::Column::ReportingPeriod.eq(period));
    }

    let models = query
        .order_by_asc(tax_withholdings_iva::Column::CreatedAt)
        .all(db)
        .await?;

    let mut result = Vec::new();
    for wh in models {
        let supplier = crate::entities::clients::Entity::find_by_id(wh.supplier_id)
            .one(db)
            .await?;
        let (supplier_rif, supplier_name) = match supplier {
            Some(s) => (s.rif.unwrap_or_default(), s.razon_social),
            None => (String::new(), "Desconocido".to_string()),
        };
        let net_payable = wh.iva_amount - wh.withheld_amount;

        let withholding_rate = wh.withholding_percentage.to_u8().ok_or_else(|| {
            AppError::Internal(format!(
                "No se pudo convertir el porcentaje de retención {} a u8 para la retención {}",
                wh.withholding_percentage, wh.id
            ))
        })?;

        result.push(IvaWithholdingResponse {
            id: wh.id,
            invoice_id: wh.invoice_id,
            supplier_rif,
            supplier_name,
            iva_amount: wh.iva_amount,
            withholding_rate,
            withheld_amount: wh.withheld_amount,
            net_payable,
            period: reconstruct_iva_period(&wh.period, &wh.reporting_period),
            voucher_number: wh.voucher_number,
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
    fn test_parse_withholding_rate_75() {
        let rate = parse_withholding_rate(75).unwrap();
        assert_eq!(rate.percentage(), dec!(75));
    }

    #[test]
    fn test_parse_withholding_rate_100() {
        let rate = parse_withholding_rate(100).unwrap();
        assert_eq!(rate.percentage(), dec!(100));
    }

    #[test]
    fn test_parse_withholding_rate_invalid() {
        assert!(parse_withholding_rate(50).is_err());
    }

    #[test]
    fn test_valid_iva_period() {
        assert!(is_valid_iva_period("202603-01"));
        assert!(is_valid_iva_period("202603-02"));
        assert!(is_valid_iva_period("202601-01"));
        assert!(is_valid_iva_period("202612-02"));
    }

    #[test]
    fn test_invalid_iva_period_old_format() {
        // Old YYYY-QQ format should be rejected
        assert!(!is_valid_iva_period("2026-01"));
        assert!(!is_valid_iva_period("2026-02"));
    }

    #[test]
    fn test_invalid_iva_period_bad_month() {
        assert!(!is_valid_iva_period("202613-01"));
        assert!(!is_valid_iva_period("202600-01"));
    }

    #[test]
    fn test_invalid_iva_period_bad_fortnight() {
        assert!(!is_valid_iva_period("202603-03"));
        assert!(!is_valid_iva_period("202603-00"));
    }

    #[test]
    fn test_invalid_iva_period_empty() {
        assert!(!is_valid_iva_period(""));
    }

    #[test]
    fn test_parse_iva_period_primera() {
        let parsed = parse_iva_period("202603-01").unwrap();
        assert_eq!(parsed.period_label, "PRIMERA");
        assert_eq!(parsed.reporting_period, "2026-03");
    }

    #[test]
    fn test_parse_iva_period_segunda() {
        let parsed = parse_iva_period("202612-02").unwrap();
        assert_eq!(parsed.period_label, "SEGUNDA");
        assert_eq!(parsed.reporting_period, "2026-12");
    }

    #[test]
    fn test_parse_iva_period_invalid() {
        assert!(parse_iva_period("2026-03").is_err());
        assert!(parse_iva_period("").is_err());
        assert!(parse_iva_period("202613-01").is_err());
    }

    #[test]
    fn test_reconstruct_iva_period_primera() {
        assert_eq!(reconstruct_iva_period("PRIMERA", "2026-03"), "202603-01");
    }

    #[test]
    fn test_reconstruct_iva_period_segunda() {
        assert_eq!(reconstruct_iva_period("SEGUNDA", "2026-12"), "202612-02");
    }

    #[test]
    fn test_roundtrip_period() {
        let original = "202601-01";
        let parsed = parse_iva_period(original).unwrap();
        let reconstructed = reconstruct_iva_period(&parsed.period_label, &parsed.reporting_period);
        assert_eq!(original, reconstructed);

        let original2 = "202603-02";
        let parsed2 = parse_iva_period(original2).unwrap();
        let reconstructed2 = reconstruct_iva_period(&parsed2.period_label, &parsed2.reporting_period);
        assert_eq!(original2, reconstructed2);
    }
}
