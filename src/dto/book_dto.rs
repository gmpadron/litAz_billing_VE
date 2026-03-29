//! DTOs de request/response para Libros de Compras y Ventas.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Purchase Book ---

/// Request para crear una entrada en el Libro de Compras.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePurchaseBookEntryRequest {
    /// Fecha de la operación de compra (YYYY-MM-DD).
    pub entry_date: NaiveDate,
    /// Nombre o razón social del proveedor.
    pub supplier_name: String,
    /// RIF del proveedor.
    pub supplier_rif: String,
    /// Número de factura del proveedor.
    pub invoice_number: String,
    /// Número de control de la factura del proveedor.
    pub control_number: String,
    /// Monto total de la factura (incluyendo IVA).
    pub total_amount: Decimal,
    /// Base imponible exenta de IVA.
    pub exempt_base: Decimal,
    /// Base imponible gravada con alícuota general (16%).
    pub general_base: Decimal,
    /// Monto del IVA general.
    pub general_tax: Decimal,
    /// Base imponible gravada con alícuota reducida (8%).
    pub reduced_base: Decimal,
    /// Monto del IVA reducido.
    pub reduced_tax: Decimal,
    /// Base imponible gravada con alicuota de lujo (31%).
    pub luxury_base: Option<Decimal>,
    /// Monto del IVA de lujo.
    pub luxury_tax: Option<Decimal>,
    /// Monto de IVA retenido al proveedor (si aplica).
    pub iva_withheld: Option<Decimal>,
    /// Periodo fiscal en formato YYYY-MM.
    pub period: String,
}

/// Response de una entrada del Libro de Compras.
#[derive(Debug, Clone, Serialize)]
pub struct PurchaseBookEntryResponse {
    pub id: Uuid,
    pub entry_date: NaiveDate,
    pub supplier_name: String,
    pub supplier_rif: String,
    pub invoice_number: String,
    pub control_number: String,
    pub total_amount: Decimal,
    pub exempt_base: Decimal,
    pub general_base: Decimal,
    pub general_tax: Decimal,
    pub reduced_base: Decimal,
    pub reduced_tax: Decimal,
    pub luxury_base: Decimal,
    pub luxury_tax: Decimal,
    pub iva_withheld: Option<Decimal>,
    pub period: String,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// --- Sales Book ---

/// Request para crear una entrada en el Libro de Ventas.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSalesBookEntryRequest {
    /// Fecha de la operación de venta (YYYY-MM-DD).
    pub entry_date: NaiveDate,
    /// Nombre o razón social del comprador.
    pub buyer_name: String,
    /// RIF del comprador (None si es consumidor final sin RIF).
    pub buyer_rif: Option<String>,
    /// Número de factura emitida.
    pub invoice_number: String,
    /// Número de control de la factura.
    pub control_number: String,
    /// Monto total de la factura (incluyendo IVA).
    pub total_amount: Decimal,
    /// Base imponible exenta de IVA.
    pub exempt_base: Decimal,
    /// Base imponible gravada con alícuota general (16%).
    pub general_base: Decimal,
    /// Monto del IVA general.
    pub general_tax: Decimal,
    /// Base imponible gravada con alícuota reducida (8%).
    pub reduced_base: Decimal,
    /// Monto del IVA reducido.
    pub reduced_tax: Decimal,
    /// Base imponible gravada con alicuota de lujo (31%).
    pub luxury_base: Option<Decimal>,
    /// Monto del IVA de lujo.
    pub luxury_tax: Option<Decimal>,
    /// Indica si es un resumen diario de consumidores finales.
    pub is_summary: Option<bool>,
    /// Periodo fiscal en formato YYYY-MM.
    pub period: String,
}

/// Response de una entrada del Libro de Ventas.
#[derive(Debug, Clone, Serialize)]
pub struct SalesBookEntryResponse {
    pub id: Uuid,
    pub entry_date: NaiveDate,
    pub buyer_name: String,
    pub buyer_rif: Option<String>,
    pub invoice_number: String,
    pub control_number: String,
    pub total_amount: Decimal,
    pub exempt_base: Decimal,
    pub general_base: Decimal,
    pub general_tax: Decimal,
    pub reduced_base: Decimal,
    pub reduced_tax: Decimal,
    pub luxury_base: Decimal,
    pub luxury_tax: Decimal,
    pub is_summary: bool,
    pub is_consumer_final: bool,
    pub period: String,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Response de resumen diario de ventas a consumidores finales.
#[derive(Debug, Clone, Serialize)]
pub struct DailySummaryResponse {
    pub date: NaiveDate,
    pub total_invoices: u32,
    pub total_amount: Decimal,
    pub exempt_base: Decimal,
    pub general_base: Decimal,
    pub general_tax: Decimal,
    pub reduced_base: Decimal,
    pub reduced_tax: Decimal,
    pub luxury_base: Decimal,
    pub luxury_tax: Decimal,
}

// --- Book totals and composite responses ---

/// Totales de un periodo del libro.
#[derive(Debug, Clone, Serialize)]
pub struct BookPeriodTotals {
    pub total_amount: Decimal,
    pub exempt_base: Decimal,
    pub general_base: Decimal,
    pub general_tax: Decimal,
    pub reduced_base: Decimal,
    pub reduced_tax: Decimal,
    pub luxury_base: Decimal,
    pub luxury_tax: Decimal,
    pub iva_withheld: Option<Decimal>,
}

/// Response completa del Libro de Compras para un periodo.
#[derive(Debug, Clone, Serialize)]
pub struct PurchaseBookResponse {
    pub period: String,
    pub entries: Vec<PurchaseBookEntryResponse>,
    pub totals: BookPeriodTotals,
}

/// Response completa del Libro de Ventas para un periodo.
#[derive(Debug, Clone, Serialize)]
pub struct SalesBookResponse {
    pub period: String,
    pub entries: Vec<SalesBookEntryResponse>,
    pub daily_summaries: Vec<DailySummaryResponse>,
    pub totals: BookPeriodTotals,
}

/// Parámetros de filtro para consultar libros fiscales.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BookFilters {
    /// Periodo fiscal en formato YYYY-MM (obligatorio para libros).
    pub period: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}
