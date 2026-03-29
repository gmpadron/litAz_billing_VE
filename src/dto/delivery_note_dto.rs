//! DTOs de request/response para Guías de Despacho / Órdenes de Entrega.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Ítem de guía de despacho en el request.
#[derive(Debug, Clone, Deserialize)]
pub struct DeliveryNoteItemRequest {
    pub description: String,
    pub quantity: Decimal,
    /// Unidad de medida (ej: "unidades", "kg", "litros").
    pub unit: String,
}

/// Request de creación de guía de despacho.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDeliveryNoteRequest {
    /// Número de factura relacionada (opcional).
    pub related_invoice_number: Option<String>,
    /// Fecha de emisión. Si no se proporciona, se usa la fecha actual.
    pub issue_date: Option<NaiveDate>,
    /// Nombre del destinatario.
    pub recipient_name: String,
    /// RIF del destinatario (opcional).
    pub recipient_rif: Option<String>,
    /// Dirección de destino.
    pub destination_address: String,
    /// Placa o identificación del vehículo (opcional).
    pub vehicle_info: Option<String>,
    /// Nombre del conductor (opcional).
    pub driver_name: Option<String>,
    /// Ítems a despachar.
    pub items: Vec<DeliveryNoteItemRequest>,
}

/// Ítem de guía de despacho en la response.
#[derive(Debug, Clone, Serialize)]
pub struct DeliveryNoteItemResponse {
    pub description: String,
    pub quantity: Decimal,
    pub unit: String,
}

/// Response de una guía de despacho.
#[derive(Debug, Clone, Serialize)]
pub struct DeliveryNoteResponse {
    pub id: Uuid,
    pub delivery_note_number: String,
    pub related_invoice_number: Option<String>,
    pub issue_date: NaiveDate,
    pub status: String,
    pub recipient_name: String,
    pub recipient_rif: Option<String>,
    pub destination_address: String,
    pub vehicle_info: Option<String>,
    pub driver_name: Option<String>,
    pub items: Vec<DeliveryNoteItemResponse>,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Respuesta resumida para listas de guías de despacho.
#[derive(Debug, Clone, Serialize)]
pub struct DeliveryNoteListResponse {
    pub id: Uuid,
    pub delivery_note_number: String,
    pub related_invoice_number: Option<String>,
    pub issue_date: NaiveDate,
    pub status: String,
    pub recipient_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Parámetros de filtro para listar guías de despacho.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DeliveryNoteFilters {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}
