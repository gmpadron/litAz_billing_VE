//! DTOs de request/response para clientes.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request de creación de cliente.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateClientRequest {
    /// RIF del cliente (opcional para consumidores finales).
    pub rif: Option<String>,
    /// Nombre o razón social.
    pub name: String,
    /// Nombre comercial (opcional).
    pub trade_name: Option<String>,
    /// Domicilio fiscal.
    pub address: String,
    /// Teléfono de contacto (opcional).
    pub phone: Option<String>,
    /// Correo electrónico (opcional).
    pub email: Option<String>,
    /// Indica si es contribuyente especial ante el SENIAT.
    pub is_special_taxpayer: Option<bool>,
}

/// Request de actualización de cliente.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateClientRequest {
    pub rif: Option<String>,
    pub name: Option<String>,
    pub trade_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_special_taxpayer: Option<bool>,
}

/// Response de un cliente.
#[derive(Debug, Clone, Serialize)]
pub struct ClientResponse {
    pub id: Uuid,
    pub rif: Option<String>,
    pub name: String,
    pub trade_name: Option<String>,
    pub address: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_special_taxpayer: bool,
    /// Número de facturas emitidas a este cliente.
    pub invoice_count: Option<u64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Respuesta resumida para listas de clientes.
#[derive(Debug, Clone, Serialize)]
pub struct ClientListResponse {
    pub id: Uuid,
    pub rif: Option<String>,
    pub name: String,
    pub trade_name: Option<String>,
    pub address: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_special_taxpayer: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Parámetros de filtro para listar clientes.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClientFilters {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    /// Búsqueda por nombre o RIF.
    pub search: Option<String>,
}
