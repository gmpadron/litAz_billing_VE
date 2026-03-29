//! Lógica de dominio para Notas de Crédito.
//!
//! Las notas de crédito referencian una factura original y disminuyen el monto adeudado.
//! No pueden superar el monto total de la factura original.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::domain::tax::iva::IvaRate;

/// Error específico de notas de crédito.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CreditNoteError {
    #[error(
        "La nota de crédito no puede exceder el monto de la factura original: máximo {max}, solicitado {requested}"
    )]
    ExceedsOriginalAmount { max: Decimal, requested: Decimal },

    #[error("La nota de crédito debe tener al menos un ítem")]
    NoItems,

    #[error("El motivo de la nota de crédito es obligatorio")]
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

/// Ítem de una nota de crédito.
#[derive(Debug, Clone)]
pub struct CreditNoteItemData {
    /// Descripción del bien o servicio devuelto/ajustado.
    pub description: String,
    /// Cantidad.
    pub quantity: Decimal,
    /// Precio unitario.
    pub unit_price: Decimal,
    /// Alícuota de IVA.
    pub tax_rate: IvaRate,
}

impl CreditNoteItemData {
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

/// Datos de una nota de crédito (entidad de dominio puro).
#[derive(Debug, Clone)]
pub struct CreditNoteData {
    /// Número de la nota de crédito.
    pub credit_note_number: String,
    /// Número de la factura original que se está corrigiendo.
    pub original_invoice_number: String,
    /// Fecha de emisión.
    pub issue_date: NaiveDate,
    /// Motivo de la nota de crédito.
    pub reason: String,
    /// Ítems de la nota de crédito.
    pub items: Vec<CreditNoteItemData>,
    /// RIF del cliente.
    pub client_rif: Option<String>,
    /// Nombre del cliente.
    pub client_name: String,
}

impl CreditNoteData {
    /// Calcula el subtotal total de la nota de crédito.
    pub fn subtotal(&self) -> Decimal {
        self.items
            .iter()
            .map(|i| i.subtotal())
            .fold(Decimal::ZERO, |acc, s| acc + s)
            .round_dp(2)
    }

    /// Calcula el IVA total de la nota de crédito.
    pub fn total_tax(&self) -> Decimal {
        self.items
            .iter()
            .map(|i| i.tax_amount())
            .fold(Decimal::ZERO, |acc, t| acc + t)
            .round_dp(2)
    }

    /// Calcula el gran total de la nota de crédito.
    pub fn grand_total(&self) -> Decimal {
        (self.subtotal() + self.total_tax()).round_dp(2)
    }
}

/// Constructor de notas de crédito con patrón Builder.
#[derive(Debug, Default)]
pub struct CreditNoteBuilder {
    credit_note_number: String,
    original_invoice_number: String,
    issue_date: Option<NaiveDate>,
    reason: String,
    items: Vec<CreditNoteItemData>,
    original_invoice_total: Option<Decimal>,
    client_rif: Option<String>,
    client_name: String,
}

impl CreditNoteBuilder {
    /// Crea un nuevo builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Establece el número de la nota de crédito.
    pub fn credit_note_number(mut self, number: impl Into<String>) -> Self {
        self.credit_note_number = number.into();
        self
    }

    /// Establece la referencia a la factura original.
    pub fn original_invoice(
        mut self,
        invoice_number: impl Into<String>,
        invoice_total: Decimal,
    ) -> Self {
        self.original_invoice_number = invoice_number.into();
        self.original_invoice_total = Some(invoice_total);
        self
    }

    /// Establece la fecha de emisión.
    pub fn issue_date(mut self, date: NaiveDate) -> Self {
        self.issue_date = Some(date);
        self
    }

    /// Establece el motivo de la nota de crédito.
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

    /// Agrega un ítem a la nota de crédito.
    pub fn add_item(
        mut self,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
        tax_rate: IvaRate,
    ) -> Self {
        self.items.push(CreditNoteItemData {
            description: description.into(),
            quantity,
            unit_price,
            tax_rate,
        });
        self
    }

    /// Construye la nota de crédito con todas las validaciones.
    pub fn build(self) -> Result<CreditNoteData, Vec<CreditNoteError>> {
        let mut errors = Vec::new();

        if self.original_invoice_number.trim().is_empty() {
            errors.push(CreditNoteError::MissingOriginalInvoiceNumber);
        }

        if self.reason.trim().is_empty() {
            errors.push(CreditNoteError::MissingReason);
        }

        if self.items.is_empty() {
            errors.push(CreditNoteError::NoItems);
        } else {
            for (i, item) in self.items.iter().enumerate() {
                if item.description.trim().is_empty() {
                    errors.push(CreditNoteError::ItemEmptyDescription(i));
                }
                if item.quantity <= Decimal::ZERO {
                    errors.push(CreditNoteError::ItemInvalidQuantity(i));
                }
                if item.unit_price < Decimal::ZERO {
                    errors.push(CreditNoteError::ItemInvalidUnitPrice(i));
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Calcular total propuesto
        let proposed_total: Decimal = self
            .items
            .iter()
            .map(|i| i.total())
            .fold(Decimal::ZERO, |acc, t| acc + t)
            .round_dp(2);

        // Verificar que no exceda el monto original
        if let Some(max) = self.original_invoice_total
            && proposed_total > max
        {
            return Err(vec![CreditNoteError::ExceedsOriginalAmount {
                max,
                requested: proposed_total,
            }]);
        }

        let date = self
            .issue_date
            .unwrap_or_else(|| chrono::Local::now().date_naive());

        Ok(CreditNoteData {
            credit_note_number: self.credit_note_number,
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
    fn test_build_valid_credit_note() {
        let note = CreditNoteBuilder::new()
            .credit_note_number("NC-0001")
            .original_invoice("00000001", dec!(116.00))
            .issue_date(sample_date())
            .reason("Devolución de mercancía")
            .client(Some("J-12345678-9"), "Empresa C.A.")
            .add_item("Producto devuelto", dec!(1), dec!(100.00), IvaRate::General)
            .build();

        assert!(note.is_ok(), "Expected Ok, got: {:?}", note.err());
    }

    #[test]
    fn test_credit_note_exceeds_original_amount() {
        let result = CreditNoteBuilder::new()
            .credit_note_number("NC-0001")
            .original_invoice("00000001", dec!(50.00))
            .issue_date(sample_date())
            .reason("Devolución")
            .client(Some("J-12345678-9"), "Empresa C.A.")
            .add_item("Producto caro", dec!(1), dec!(200.00), IvaRate::General)
            .build();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err()[0],
            CreditNoteError::ExceedsOriginalAmount { .. }
        ));
    }

    #[test]
    fn test_credit_note_missing_reason() {
        let result = CreditNoteBuilder::new()
            .credit_note_number("NC-0001")
            .original_invoice("00000001", dec!(200.00))
            .issue_date(sample_date())
            .reason("")
            .client(Some("J-12345678-9"), "Empresa C.A.")
            .add_item("Producto", dec!(1), dec!(100.00), IvaRate::General)
            .build();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains(&CreditNoteError::MissingReason)
        );
    }

    #[test]
    fn test_credit_note_no_items() {
        let result = CreditNoteBuilder::new()
            .credit_note_number("NC-0001")
            .original_invoice("00000001", dec!(200.00))
            .issue_date(sample_date())
            .reason("Devolución")
            .client(Some("J-12345678-9"), "Empresa C.A.")
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains(&CreditNoteError::NoItems));
    }

    #[test]
    fn test_credit_note_totals() {
        let note = CreditNoteBuilder::new()
            .credit_note_number("NC-0001")
            .original_invoice("00000001", dec!(232.00))
            .issue_date(sample_date())
            .reason("Devolución parcial")
            .client(Some("J-12345678-9"), "Empresa C.A.")
            .add_item("Producto 1", dec!(1), dec!(100.00), IvaRate::General)
            .add_item("Producto 2", dec!(1), dec!(100.00), IvaRate::General)
            .build()
            .unwrap();

        assert_eq!(note.subtotal(), dec!(200.00));
        assert_eq!(note.total_tax(), dec!(32.00));
        assert_eq!(note.grand_total(), dec!(232.00));
    }
}
