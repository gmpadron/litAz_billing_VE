//! Lógica de dominio para Notas de Débito.
//!
//! Las notas de débito referencian una factura original y agregan cargos adicionales.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::domain::tax::iva::IvaRate;

/// Error específico de notas de débito.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DebitNoteError {
    #[error("La nota de débito debe tener al menos un ítem")]
    NoItems,

    #[error("El motivo de la nota de débito es obligatorio")]
    MissingReason,

    #[error("El número de la factura original es obligatorio")]
    MissingOriginalInvoiceNumber,

    #[error("El ítem {0} tiene cantidad inválida (debe ser > 0)")]
    ItemInvalidQuantity(usize),

    #[error("El ítem {0} tiene precio unitario inválido (debe ser >= 0)")]
    ItemInvalidUnitPrice(usize),

    #[error("El ítem {0} tiene descripción vacía")]
    ItemEmptyDescription(usize),
}

/// Ítem de una nota de débito.
#[derive(Debug, Clone)]
pub struct DebitNoteItemData {
    /// Descripción del cargo adicional.
    pub description: String,
    /// Cantidad.
    pub quantity: Decimal,
    /// Precio unitario.
    pub unit_price: Decimal,
    /// Alícuota de IVA.
    pub tax_rate: IvaRate,
}

impl DebitNoteItemData {
    /// Calcula el subtotal del ítem.
    pub fn subtotal(&self) -> Decimal {
        self.quantity * self.unit_price
    }

    /// Calcula el monto de IVA del ítem.
    pub fn tax_amount(&self) -> Decimal {
        (self.subtotal() * self.tax_rate.rate()).round_dp(2)
    }

    /// Calcula el total del ítem (subtotal + IVA).
    pub fn total(&self) -> Decimal {
        (self.subtotal() + self.tax_amount()).round_dp(2)
    }
}

/// Datos de una nota de débito (entidad de dominio puro).
#[derive(Debug, Clone)]
pub struct DebitNoteData {
    /// Número de la nota de débito.
    pub debit_note_number: String,
    /// Número de la factura original que se está complementando.
    pub original_invoice_number: String,
    /// Fecha de emisión.
    pub issue_date: NaiveDate,
    /// Motivo de la nota de débito.
    pub reason: String,
    /// Ítems de la nota de débito.
    pub items: Vec<DebitNoteItemData>,
    /// RIF del cliente.
    pub client_rif: Option<String>,
    /// Nombre del cliente.
    pub client_name: String,
}

impl DebitNoteData {
    /// Calcula el subtotal total de la nota de débito.
    pub fn subtotal(&self) -> Decimal {
        self.items
            .iter()
            .map(|i| i.subtotal())
            .fold(Decimal::ZERO, |acc, s| acc + s)
            .round_dp(2)
    }

    /// Calcula el IVA total de la nota de débito.
    pub fn total_tax(&self) -> Decimal {
        self.items
            .iter()
            .map(|i| i.tax_amount())
            .fold(Decimal::ZERO, |acc, t| acc + t)
            .round_dp(2)
    }

    /// Calcula el gran total de la nota de débito.
    pub fn grand_total(&self) -> Decimal {
        (self.subtotal() + self.total_tax()).round_dp(2)
    }
}

/// Constructor de notas de débito con patrón Builder.
#[derive(Debug, Default)]
pub struct DebitNoteBuilder {
    debit_note_number: String,
    original_invoice_number: String,
    issue_date: Option<NaiveDate>,
    reason: String,
    items: Vec<DebitNoteItemData>,
    client_rif: Option<String>,
    client_name: String,
}

impl DebitNoteBuilder {
    /// Crea un nuevo builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Establece el número de la nota de débito.
    pub fn debit_note_number(mut self, number: impl Into<String>) -> Self {
        self.debit_note_number = number.into();
        self
    }

    /// Establece la referencia a la factura original.
    pub fn original_invoice(mut self, invoice_number: impl Into<String>) -> Self {
        self.original_invoice_number = invoice_number.into();
        self
    }

    /// Establece la fecha de emisión.
    pub fn issue_date(mut self, date: NaiveDate) -> Self {
        self.issue_date = Some(date);
        self
    }

    /// Establece el motivo de la nota de débito.
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    /// Establece los datos del cliente.
    pub fn client(mut self, rif: Option<impl Into<String>>, name: impl Into<String>) -> Self {
        self.client_rif = rif.map(|r| r.into());
        self.client_name = name.into();
        self
    }

    /// Agrega un ítem a la nota de débito.
    pub fn add_item(
        mut self,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
        tax_rate: IvaRate,
    ) -> Self {
        self.items.push(DebitNoteItemData {
            description: description.into(),
            quantity,
            unit_price,
            tax_rate,
        });
        self
    }

    /// Construye la nota de débito con todas las validaciones.
    pub fn build(self) -> Result<DebitNoteData, Vec<DebitNoteError>> {
        let mut errors = Vec::new();

        if self.original_invoice_number.trim().is_empty() {
            errors.push(DebitNoteError::MissingOriginalInvoiceNumber);
        }

        if self.reason.trim().is_empty() {
            errors.push(DebitNoteError::MissingReason);
        }

        if self.items.is_empty() {
            errors.push(DebitNoteError::NoItems);
        } else {
            for (i, item) in self.items.iter().enumerate() {
                if item.description.trim().is_empty() {
                    errors.push(DebitNoteError::ItemEmptyDescription(i));
                }
                if item.quantity <= Decimal::ZERO {
                    errors.push(DebitNoteError::ItemInvalidQuantity(i));
                }
                if item.unit_price < Decimal::ZERO {
                    errors.push(DebitNoteError::ItemInvalidUnitPrice(i));
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let date = self
            .issue_date
            .unwrap_or_else(|| chrono::Local::now().date_naive());

        Ok(DebitNoteData {
            debit_note_number: self.debit_note_number,
            original_invoice_number: self.original_invoice_number,
            issue_date: date,
            reason: self.reason,
            items: self.items,
            client_rif: self.client_rif,
            client_name: self.client_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, 26).unwrap()
    }

    #[test]
    fn test_build_valid_debit_note() {
        let note = DebitNoteBuilder::new()
            .debit_note_number("ND-0001")
            .original_invoice("00000001")
            .issue_date(sample_date())
            .reason("Cargo por flete adicional")
            .client(Some("J-12345678-9"), "Empresa C.A.")
            .add_item("Flete", dec!(1), dec!(50.00), IvaRate::General)
            .build();

        assert!(note.is_ok(), "Expected Ok, got: {:?}", note.err());
    }

    #[test]
    fn test_debit_note_missing_reason() {
        let result = DebitNoteBuilder::new()
            .debit_note_number("ND-0001")
            .original_invoice("00000001")
            .issue_date(sample_date())
            .reason("")
            .client(Some("J-12345678-9"), "Empresa C.A.")
            .add_item("Flete", dec!(1), dec!(50.00), IvaRate::General)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains(&DebitNoteError::MissingReason));
    }

    #[test]
    fn test_debit_note_no_items() {
        let result = DebitNoteBuilder::new()
            .debit_note_number("ND-0001")
            .original_invoice("00000001")
            .issue_date(sample_date())
            .reason("Cargo adicional")
            .client(Some("J-12345678-9"), "Empresa C.A.")
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains(&DebitNoteError::NoItems));
    }

    #[test]
    fn test_debit_note_totals() {
        let note = DebitNoteBuilder::new()
            .debit_note_number("ND-0001")
            .original_invoice("00000001")
            .issue_date(sample_date())
            .reason("Cargos adicionales")
            .client(Some("J-12345678-9"), "Empresa C.A.")
            .add_item("Flete", dec!(1), dec!(50.00), IvaRate::General)
            .add_item("Seguro", dec!(1), dec!(20.00), IvaRate::Exempt)
            .build()
            .unwrap();

        assert_eq!(note.subtotal(), dec!(70.00));
        assert_eq!(note.total_tax(), dec!(8.00));
        assert_eq!(note.grand_total(), dec!(78.00));
    }
}
