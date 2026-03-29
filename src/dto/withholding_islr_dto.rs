//! DTOs de request/response para retenciones de ISLR.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request para registrar una retención de ISLR.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateIslrWithholdingRequest {
    /// ID de la factura asociada.
    pub invoice_id: Uuid,
    /// RIF del beneficiario (proveedor/prestador de servicio).
    pub beneficiary_rif: String,
    /// Razón social del beneficiario.
    pub beneficiary_name: String,
    /// Monto base sobre el cual se calcula la retención.
    pub base_amount: Decimal,
    /// Tipo de actividad (ej: "servicios_profesionales", "alquiler", etc.).
    pub activity_type: String,
    /// Tasa de retención como porcentaje (ej: 5 para 5%). Se convierte a fracción decimal internamente.
    pub withholding_rate: Decimal,
    /// Sustraendo a aplicar (opcional, por defecto 0).
    pub subtract_amount: Option<Decimal>,
    /// Periodo fiscal mensual en formato YYYY-MM (ej: "2026-03").
    pub period: String,
}

/// Response de una retención de ISLR.
#[derive(Debug, Clone, Serialize)]
pub struct IslrWithholdingResponse {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub beneficiary_rif: String,
    pub beneficiary_name: String,
    pub base_amount: Decimal,
    pub activity_type: String,
    pub withholding_rate: Decimal,
    pub subtract_amount: Decimal,
    pub withheld_amount: Decimal,
    pub net_payable: Decimal,
    pub period: String,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Respuesta resumida para listas de retenciones ISLR.
#[derive(Debug, Clone, Serialize)]
pub struct IslrWithholdingListResponse {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub beneficiary_rif: String,
    pub beneficiary_name: String,
    pub base_amount: Decimal,
    pub withheld_amount: Decimal,
    pub period: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Parámetros de filtro para listar retenciones de ISLR.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct IslrWithholdingFilters {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub period: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}
