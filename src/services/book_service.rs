//! Servicio de Libros de Compras y Ventas: persistencia real con SeaORM.

use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::domain::books::{PurchaseBookEntry, SalesBookEntry};
use crate::dto::book_dto::{
    BookFilters, BookPeriodTotals, CreatePurchaseBookEntryRequest, CreateSalesBookEntryRequest,
    DailySummaryResponse, PurchaseBookEntryResponse, PurchaseBookResponse, SalesBookEntryResponse,
    SalesBookResponse,
};
use crate::entities::{purchase_book_entries, sales_book_entries};
use crate::errors::AppError;

/// Valida que el periodo tenga formato YYYY-MM.
fn validate_period(period: &str) -> Result<(), AppError> {
    let parts: Vec<&str> = period.split('-').collect();
    if parts.len() != 2 {
        return Err(AppError::BadRequest(format!(
            "Formato de periodo inválido: '{}'. Esperado: YYYY-MM",
            period
        )));
    }

    let year: Result<u32, _> = parts[0].parse();
    let month: Result<u32, _> = parts[1].parse();

    match (year, month) {
        (Ok(y), Ok(m)) if y >= 2000 && y <= 9999 && (1..=12).contains(&m) => Ok(()),
        _ => Err(AppError::BadRequest(format!(
            "Formato de periodo inválido: '{}'. Esperado: YYYY-MM",
            period
        ))),
    }
}

/// Crea una entrada en el Libro de Compras con persistencia real.
pub async fn create_purchase_book_entry(
    db: &DatabaseConnection,
    dto: CreatePurchaseBookEntryRequest,
    user_id: Uuid,
    company_profile_id: Uuid,
) -> Result<PurchaseBookEntryResponse, AppError> {
    let entry = PurchaseBookEntry {
        entry_date: dto.entry_date,
        supplier_name: dto.supplier_name.clone(),
        supplier_rif: dto.supplier_rif.clone(),
        invoice_number: dto.invoice_number.clone(),
        control_number: dto.control_number.clone(),
        total_amount: dto.total_amount,
        exempt_base: dto.exempt_base,
        general_base: dto.general_base,
        general_tax: dto.general_tax,
        reduced_base: dto.reduced_base,
        reduced_tax: dto.reduced_tax,
        luxury_base: dto.luxury_base.unwrap_or(Decimal::ZERO),
        luxury_tax: dto.luxury_tax.unwrap_or(Decimal::ZERO),
        iva_withheld: dto.iva_withheld,
        period: dto.period.clone(),
    };

    entry
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let id = Uuid::new_v4();
    let now = Utc::now().into();

    let model = purchase_book_entries::ActiveModel {
        id: Set(id),
        company_profile_id: Set(company_profile_id),
        period: Set(dto.period.clone()),
        entry_date: Set(now),
        supplier_rif: Set(dto.supplier_rif.clone()),
        supplier_name: Set(dto.supplier_name.clone()),
        invoice_number: Set(dto.invoice_number.clone()),
        control_number: Set(dto.control_number.clone()),
        invoice_date: Set(dto.entry_date),
        total_amount: Set(dto.total_amount),
        base_imponible_exenta: Set(dto.exempt_base),
        base_imponible_reducida: Set(dto.reduced_base),
        base_imponible_general: Set(dto.general_base),
        base_imponible_lujo: Set(dto.luxury_base.unwrap_or(Decimal::ZERO)),
        iva_reducida: Set(dto.reduced_tax),
        iva_general: Set(dto.general_tax),
        iva_lujo: Set(dto.luxury_tax.unwrap_or(Decimal::ZERO)),
        iva_retenido: Set(dto.iva_withheld.unwrap_or(Decimal::ZERO)),
        operation_type: Set("COMPRA".to_string()),
        invoice_id: Set(None),
        created_by: Set(user_id),
        created_at: Set(now),
        updated_at: Set(now),
    };

    model.insert(db).await?;

    Ok(PurchaseBookEntryResponse {
        id,
        entry_date: dto.entry_date,
        supplier_name: dto.supplier_name,
        supplier_rif: dto.supplier_rif,
        invoice_number: dto.invoice_number,
        control_number: dto.control_number,
        total_amount: dto.total_amount,
        exempt_base: dto.exempt_base,
        general_base: dto.general_base,
        general_tax: dto.general_tax,
        reduced_base: dto.reduced_base,
        reduced_tax: dto.reduced_tax,
        luxury_base: dto.luxury_base.unwrap_or(Decimal::ZERO),
        luxury_tax: dto.luxury_tax.unwrap_or(Decimal::ZERO),
        iva_withheld: dto.iva_withheld,
        period: dto.period,
        created_by: user_id,
        created_at: Utc::now(),
    })
}

/// Crea una entrada en el Libro de Ventas con persistencia real.
pub async fn create_sales_book_entry(
    db: &DatabaseConnection,
    dto: CreateSalesBookEntryRequest,
    user_id: Uuid,
    company_profile_id: Uuid,
) -> Result<SalesBookEntryResponse, AppError> {
    let is_summary = dto.is_summary.unwrap_or(false);

    let entry = SalesBookEntry {
        entry_date: dto.entry_date,
        buyer_name: dto.buyer_name.clone(),
        buyer_rif: dto.buyer_rif.clone(),
        invoice_number: dto.invoice_number.clone(),
        control_number: dto.control_number.clone(),
        total_amount: dto.total_amount,
        exempt_base: dto.exempt_base,
        general_base: dto.general_base,
        general_tax: dto.general_tax,
        reduced_base: dto.reduced_base,
        reduced_tax: dto.reduced_tax,
        luxury_base: dto.luxury_base.unwrap_or(Decimal::ZERO),
        luxury_tax: dto.luxury_tax.unwrap_or(Decimal::ZERO),
        is_summary,
        period: dto.period.clone(),
    };

    entry
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let is_consumer_final = entry.is_consumer_final();

    let id = Uuid::new_v4();
    let now = Utc::now().into();

    let model = sales_book_entries::ActiveModel {
        id: Set(id),
        company_profile_id: Set(company_profile_id),
        period: Set(dto.period.clone()),
        entry_date: Set(now),
        client_rif: Set(dto.buyer_rif.clone()),
        client_name: Set(dto.buyer_name.clone()),
        invoice_number: Set(dto.invoice_number.clone()),
        control_number: Set(dto.control_number.clone()),
        invoice_date: Set(dto.entry_date),
        total_amount: Set(dto.total_amount),
        base_imponible_exenta: Set(dto.exempt_base),
        base_imponible_reducida: Set(dto.reduced_base),
        base_imponible_general: Set(dto.general_base),
        base_imponible_lujo: Set(dto.luxury_base.unwrap_or(Decimal::ZERO)),
        iva_reducida: Set(dto.reduced_tax),
        iva_general: Set(dto.general_tax),
        iva_lujo: Set(dto.luxury_tax.unwrap_or(Decimal::ZERO)),
        es_resumen_diario: Set(is_summary),
        invoice_id: Set(None),
        created_by: Set(user_id),
        created_at: Set(now),
        updated_at: Set(now),
    };

    model.insert(db).await?;

    Ok(SalesBookEntryResponse {
        id,
        entry_date: dto.entry_date,
        buyer_name: dto.buyer_name,
        buyer_rif: dto.buyer_rif,
        invoice_number: dto.invoice_number,
        control_number: dto.control_number,
        total_amount: dto.total_amount,
        exempt_base: dto.exempt_base,
        general_base: dto.general_base,
        general_tax: dto.general_tax,
        reduced_base: dto.reduced_base,
        reduced_tax: dto.reduced_tax,
        luxury_base: dto.luxury_base.unwrap_or(Decimal::ZERO),
        luxury_tax: dto.luxury_tax.unwrap_or(Decimal::ZERO),
        is_summary,
        is_consumer_final,
        period: dto.period,
        created_by: user_id,
        created_at: Utc::now(),
    })
}

/// Obtiene el Libro de Compras para un periodo mensual.
pub async fn get_purchase_book(
    db: &DatabaseConnection,
    filters: BookFilters,
) -> Result<PurchaseBookResponse, AppError> {
    let period = filters.period.ok_or_else(|| {
        AppError::BadRequest("El parámetro 'period' es obligatorio (formato: YYYY-MM)".to_string())
    })?;

    validate_period(&period)?;

    let models = purchase_book_entries::Entity::find()
        .filter(purchase_book_entries::Column::Period.eq(period.as_str()))
        .order_by_asc(purchase_book_entries::Column::EntryDate)
        .all(db)
        .await?;

    let mut totals = BookPeriodTotals {
        total_amount: Decimal::ZERO,
        exempt_base: Decimal::ZERO,
        general_base: Decimal::ZERO,
        general_tax: Decimal::ZERO,
        reduced_base: Decimal::ZERO,
        reduced_tax: Decimal::ZERO,
        luxury_base: Decimal::ZERO,
        luxury_tax: Decimal::ZERO,
        iva_withheld: Some(Decimal::ZERO),
    };

    let entries: Vec<PurchaseBookEntryResponse> = models
        .into_iter()
        .map(|m| {
            totals.total_amount += m.total_amount;
            totals.exempt_base += m.base_imponible_exenta;
            totals.general_base += m.base_imponible_general;
            totals.general_tax += m.iva_general;
            totals.reduced_base += m.base_imponible_reducida;
            totals.reduced_tax += m.iva_reducida;
            totals.luxury_base += m.base_imponible_lujo;
            totals.luxury_tax += m.iva_lujo;
            if let Some(ref mut withheld) = totals.iva_withheld {
                *withheld += m.iva_retenido;
            }

            PurchaseBookEntryResponse {
                id: m.id,
                entry_date: m.invoice_date,
                supplier_name: m.supplier_name,
                supplier_rif: m.supplier_rif,
                invoice_number: m.invoice_number,
                control_number: m.control_number,
                total_amount: m.total_amount,
                exempt_base: m.base_imponible_exenta,
                general_base: m.base_imponible_general,
                general_tax: m.iva_general,
                reduced_base: m.base_imponible_reducida,
                reduced_tax: m.iva_reducida,
                luxury_base: m.base_imponible_lujo,
                luxury_tax: m.iva_lujo,
                iva_withheld: Some(m.iva_retenido),
                period: m.period,
                created_by: m.created_by,
                created_at: m.created_at.with_timezone(&Utc),
            }
        })
        .collect();

    Ok(PurchaseBookResponse {
        period,
        entries,
        totals,
    })
}

/// Obtiene el Libro de Ventas para un periodo mensual.
pub async fn get_sales_book(
    db: &DatabaseConnection,
    filters: BookFilters,
) -> Result<SalesBookResponse, AppError> {
    let period = filters.period.ok_or_else(|| {
        AppError::BadRequest("El parámetro 'period' es obligatorio (formato: YYYY-MM)".to_string())
    })?;

    validate_period(&period)?;

    let models = sales_book_entries::Entity::find()
        .filter(sales_book_entries::Column::Period.eq(period.as_str()))
        .order_by_asc(sales_book_entries::Column::EntryDate)
        .all(db)
        .await?;

    let mut totals = BookPeriodTotals {
        total_amount: Decimal::ZERO,
        exempt_base: Decimal::ZERO,
        general_base: Decimal::ZERO,
        general_tax: Decimal::ZERO,
        reduced_base: Decimal::ZERO,
        reduced_tax: Decimal::ZERO,
        luxury_base: Decimal::ZERO,
        luxury_tax: Decimal::ZERO,
        iva_withheld: None,
    };

    // Build daily summaries for consumer final entries
    use std::collections::BTreeMap;
    let mut daily_map: BTreeMap<chrono::NaiveDate, DailySummaryResponse> = BTreeMap::new();

    let entries: Vec<SalesBookEntryResponse> = models
        .into_iter()
        .map(|m| {
            totals.total_amount += m.total_amount;
            totals.exempt_base += m.base_imponible_exenta;
            totals.general_base += m.base_imponible_general;
            totals.general_tax += m.iva_general;
            totals.reduced_base += m.base_imponible_reducida;
            totals.reduced_tax += m.iva_reducida;
            totals.luxury_base += m.base_imponible_lujo;
            totals.luxury_tax += m.iva_lujo;

            let is_consumer_final = m.client_rif.is_none();

            if is_consumer_final {
                let date = m.invoice_date;
                let summary = daily_map.entry(date).or_insert(DailySummaryResponse {
                    date,
                    total_invoices: 0,
                    total_amount: Decimal::ZERO,
                    exempt_base: Decimal::ZERO,
                    general_base: Decimal::ZERO,
                    general_tax: Decimal::ZERO,
                    reduced_base: Decimal::ZERO,
                    reduced_tax: Decimal::ZERO,
                    luxury_base: Decimal::ZERO,
                    luxury_tax: Decimal::ZERO,
                });
                summary.total_invoices += 1;
                summary.total_amount += m.total_amount;
                summary.exempt_base += m.base_imponible_exenta;
                summary.general_base += m.base_imponible_general;
                summary.general_tax += m.iva_general;
                summary.reduced_base += m.base_imponible_reducida;
                summary.reduced_tax += m.iva_reducida;
                summary.luxury_base += m.base_imponible_lujo;
                summary.luxury_tax += m.iva_lujo;
            }

            SalesBookEntryResponse {
                id: m.id,
                entry_date: m.invoice_date,
                buyer_name: m.client_name,
                buyer_rif: m.client_rif,
                invoice_number: m.invoice_number,
                control_number: m.control_number,
                total_amount: m.total_amount,
                exempt_base: m.base_imponible_exenta,
                general_base: m.base_imponible_general,
                general_tax: m.iva_general,
                reduced_base: m.base_imponible_reducida,
                reduced_tax: m.iva_reducida,
                luxury_base: m.base_imponible_lujo,
                luxury_tax: m.iva_lujo,
                is_summary: m.es_resumen_diario,
                is_consumer_final,
                period: m.period,
                created_by: m.created_by,
                created_at: m.created_at.with_timezone(&Utc),
            }
        })
        .collect();

    let daily_summaries: Vec<DailySummaryResponse> = daily_map.into_values().collect();

    Ok(SalesBookResponse {
        period,
        entries,
        daily_summaries,
        totals,
    })
}
