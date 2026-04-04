//! DTOs de request/response para perfiles fiscales de empresa (multi-empresa).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Item resumido para la lista de empresas (GET /company).
#[derive(Debug, Clone, Serialize)]
pub struct CompanyListItem {
    pub id: Uuid,
    pub business_name: String,
    pub trade_name: Option<String>,
    pub rif: String,
    pub is_special_taxpayer: bool,
    pub is_active: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Response completo de una empresa.
#[derive(Debug, Clone, Serialize)]
pub struct CompanyResponse {
    pub id: Uuid,
    pub business_name: String,
    pub trade_name: Option<String>,
    pub rif: String,
    pub fiscal_address: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_special_taxpayer: bool,
    pub special_taxpayer_resolution: Option<String>,
    pub is_active: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request para CREAR una empresa (POST /company).
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCompanyRequest {
    /// Razón social — requerido.
    pub business_name: String,
    /// RIF — requerido, único en el sistema.
    pub rif: String,
    pub trade_name: Option<String>,
    pub fiscal_address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_special_taxpayer: Option<bool>,
    pub special_taxpayer_resolution: Option<String>,
}

/// Request para ACTUALIZAR una empresa (PUT /company/{id}).
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
