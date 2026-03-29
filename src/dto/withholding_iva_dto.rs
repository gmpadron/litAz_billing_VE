//! DTOs de request/response para retenciones de IVA.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request para registrar una retención de IVA.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateIvaWithholdingRequest {
    /// ID de la factura asociada.
    pub invoice_id: Uuid,
    /// RIF del proveedor al que se le retiene.
    pub supplier_rif: String,
    /// Razón social del proveedor.
    pub supplier_name: String,
    /// Monto del IVA facturado (debe ser >= 0).
    pub iva_amount: Decimal,
    /// Porcentaje de retención: 75 o 100.
    pub withholding_rate: u8,
    /// Periodo fiscal quincenal en formato YYYYMM-QQ (ej: "202601-01" para primera quincena enero,
    /// "202603-02" para segunda quincena marzo). Este formato es el requerido por el SENIAT en el XML.
    pub period: String,
    /// Número de comprobante de retención.
    pub voucher_number: String,
}

/// Response de una retención de IVA.
#[derive(Debug, Clone, Serialize)]
pub struct IvaWithholdingResponse {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub supplier_rif: String,
    pub supplier_name: String,
    pub iva_amount: Decimal,
    pub withholding_rate: u8,
    pub withheld_amount: Decimal,
    pub net_payable: Decimal,
    pub period: String,
    pub voucher_number: String,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Respuesta resumida para listas de retenciones IVA.
#[derive(Debug, Clone, Serialize)]
pub struct IvaWithholdingListResponse {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub supplier_rif: String,
    pub supplier_name: String,
    pub iva_amount: Decimal,
    pub withheld_amount: Decimal,
    pub period: String,
    pub voucher_number: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Parámetros de filtro para listar retenciones de IVA.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct IvaWithholdingFilters {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub period: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}
