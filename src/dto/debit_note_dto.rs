//! DTOs de request/response para Notas de Débito.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Ítem de nota de débito en el request.
#[derive(Debug, Clone, Deserialize)]
pub struct DebitNoteItemRequest {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    /// Alícuota de IVA: "general", "reduced", "luxury", "exempt".
    pub tax_rate: String,
}

/// Request de creación de nota de débito.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDebitNoteRequest {
    /// Número de la factura original que se está complementando.
    pub original_invoice_number: String,
    /// Fecha de emisión. Si no se proporciona, se usa la fecha actual.
    pub issue_date: Option<NaiveDate>,
    /// Motivo de la nota de débito.
    pub reason: String,
    /// Ítems de la nota de débito.
    pub items: Vec<DebitNoteItemRequest>,
}

/// Ítem de nota de débito en la response.
#[derive(Debug, Clone, Serialize)]
pub struct DebitNoteItemResponse {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub tax_rate: String,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
}

/// Response de una nota de débito.
#[derive(Debug, Clone, Serialize)]
pub struct DebitNoteResponse {
    pub id: Uuid,
    pub debit_note_number: String,
    pub original_invoice_number: String,
    pub issue_date: NaiveDate,
    pub status: String,
    pub reason: String,
    pub client_rif: Option<String>,
    pub client_name: String,
    pub items: Vec<DebitNoteItemResponse>,
    pub subtotal: Decimal,
    pub total_tax: Decimal,
    pub grand_total: Decimal,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Respuesta resumida para listas de notas de débito.
#[derive(Debug, Clone, Serialize)]
pub struct DebitNoteListResponse {
    pub id: Uuid,
    pub debit_note_number: String,
    pub original_invoice_number: String,
    pub issue_date: NaiveDate,
    pub status: String,
    pub reason: String,
    pub grand_total: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Parámetros de filtro para listar notas de débito.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DebitNoteFilters {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}
