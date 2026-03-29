//! DTOs de request/response para el perfil fiscal de la empresa emisora.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response del perfil fiscal de la empresa.
#[derive(Debug, Clone, Serialize)]
pub struct CompanyResponse {
    pub id: Uuid,
    /// Razón social de la empresa.
    pub business_name: String,
    /// Nombre comercial (puede diferir de la razón social).
    pub trade_name: Option<String>,
    /// RIF de la empresa emisora.
    pub rif: String,
    /// Domicilio fiscal.
    pub fiscal_address: String,
    /// Teléfono de contacto.
    pub phone: Option<String>,
    /// Correo electrónico.
    pub email: Option<String>,
    /// Indica si es contribuyente especial ante el SENIAT.
    pub is_special_taxpayer: bool,
    /// Número de resolución de contribuyente especial (si aplica).
    pub special_taxpayer_resolution: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request de actualización del perfil fiscal de la empresa.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCompanyRequest {
    pub business_name: Option<String>,
    pub trade_name: Option<String>,
    pub rif: Option<String>,
    pub fiscal_address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_special_taxpayer: Option<bool>,
    pub special_taxpayer_resolution: Option<String>,
}
