//! DTOs de request/response para facturas.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Ítem de factura en el request de creación.
#[derive(Debug, Clone, Deserialize)]
pub struct InvoiceItemRequest {
    /// Descripción del bien o servicio.
    pub description: String,
    /// Cantidad (debe ser > 0).
    pub quantity: Decimal,
    /// Precio unitario (debe ser >= 0).
    pub unit_price: Decimal,
    /// Alícuota de IVA: "general" (16%), "reduced" (8%), "luxury" (31%), "exempt" (0%).
    pub tax_rate: String,
}

/// Request de creación de factura.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateInvoiceRequest {
    /// Fecha de emisión (YYYY-MM-DD). Si no se proporciona, se usa la fecha actual.
    pub invoice_date: Option<NaiveDate>,
    /// RIF del cliente (None si es consumidor final).
    pub client_rif: Option<String>,
    /// Nombre o razón social del cliente.
    pub client_name: String,
    /// Domicilio fiscal del cliente.
    #[serde(default)]
    pub client_address: Option<String>,
    /// Ítems de la factura.
    pub items: Vec<InvoiceItemRequest>,
    /// Condición de pago: "cash" o "credit".
    pub payment_condition: String,
    /// Plazo en días para crédito (requerido si payment_condition es "credit").
    pub credit_days: Option<u32>,
    /// Moneda de la operación (ej: "USD", "VES").
    pub currency: String,
    /// Tasa de cambio BCV del día.
    pub exchange_rate: Decimal,
}

/// Ítem de factura en la response.
#[derive(Debug, Clone, Serialize)]
pub struct InvoiceItemResponse {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub tax_rate: String,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
}

/// Response de una factura.
#[derive(Debug, Clone, Serialize)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub invoice_number: String,
    pub control_number: String,
    pub invoice_date: NaiveDate,
    pub status: String,
    pub client_rif: Option<String>,
    pub client_name: String,
    pub client_address: String,
    pub items: Vec<InvoiceItemResponse>,
    pub payment_condition: String,
    pub credit_days: Option<u32>,
    pub no_fiscal_credit: bool,
    pub currency: String,
    pub exchange_rate: Decimal,
    /// Subtotal (sin IVA).
    pub subtotal: Decimal,
    /// IVA a alícuota general (16%).
    pub tax_general: Decimal,
    /// IVA a alícuota reducida (8%).
    pub tax_reduced: Decimal,
    /// IVA a alícuota de lujo (31%).
    pub tax_luxury: Decimal,
    /// Total de IVA.
    pub total_tax: Decimal,
    /// Gran total (subtotal + IVA).
    pub grand_total: Decimal,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Respuesta resumida para listas de facturas.
#[derive(Debug, Clone, Serialize)]
pub struct InvoiceListResponse {
    pub id: Uuid,
    pub invoice_number: String,
    pub control_number: String,
    pub invoice_date: NaiveDate,
    pub status: String,
    pub client_name: String,
    pub client_rif: Option<String>,
    pub grand_total: Decimal,
    pub currency: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Parámetros de filtro para listar facturas.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct InvoiceFilters {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub status: Option<String>,
}

/// Request para anular una factura.
#[derive(Debug, Clone, Deserialize)]
pub struct VoidInvoiceRequest {
    /// Motivo de la anulación.
    pub reason: String,
}
