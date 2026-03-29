//! DTOs de request/response para Notas de Crédito.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Ítem de nota de crédito en el request.
#[derive(Debug, Clone, Deserialize)]
pub struct CreditNoteItemRequest {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    /// Alícuota de IVA: "general", "reduced", "luxury", "exempt".
    pub tax_rate: String,
}

/// Request de creación de nota de crédito.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCreditNoteRequest {
    /// Número de la factura original que se está corrigiendo.
    pub original_invoice_number: String,
    /// Fecha de emisión. Si no se proporciona, se usa la fecha actual.
    pub issue_date: Option<NaiveDate>,
    /// Motivo de la nota de crédito.
    pub reason: String,
    /// Ítems de la nota de crédito.
    pub items: Vec<CreditNoteItemRequest>,
}

/// Ítem de nota de crédito en la response.
#[derive(Debug, Clone, Serialize)]
pub struct CreditNoteItemResponse {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub tax_rate: String,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
}

/// Response de una nota de crédito.
#[derive(Debug, Clone, Serialize)]
pub struct CreditNoteResponse {
    pub id: Uuid,
    pub credit_note_number: String,
    pub original_invoice_number: String,
    pub issue_date: NaiveDate,
    pub status: String,
    pub reason: String,
    pub client_rif: Option<String>,
    pub client_name: String,
    pub items: Vec<CreditNoteItemResponse>,
    pub subtotal: Decimal,
    pub total_tax: Decimal,
    pub grand_total: Decimal,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Respuesta resumida para listas de notas de crédito.
#[derive(Debug, Clone, Serialize)]
pub struct CreditNoteListResponse {
    pub id: Uuid,
    pub credit_note_number: String,
    pub original_invoice_number: String,
    pub issue_date: NaiveDate,
    pub status: String,
    pub reason: String,
    pub grand_total: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Parámetros de filtro para listar notas de crédito.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CreditNoteFilters {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}
