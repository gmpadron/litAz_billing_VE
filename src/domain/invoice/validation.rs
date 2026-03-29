//! Validaciones de datos obligatorios para documentos fiscales según SENIAT (PA 0071).

use thiserror::Error;

use super::builder::InvoiceData;

/// Error de validación de datos de factura.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("El número de factura es obligatorio")]
    MissingInvoiceNumber,

    #[error("El número de control es obligatorio")]
    MissingControlNumber,

    #[error("La fecha de emisión es obligatoria")]
    MissingInvoiceDate,

    #[error("El nombre o razón social del cliente es obligatorio")]
    MissingClientName,

    #[error("El domicilio fiscal del cliente es obligatorio")]
    MissingClientAddress,

    #[error("La factura debe tener al menos un ítem")]
    NoItems,

    #[error("El ítem '{0}' tiene descripción vacía")]
    ItemEmptyDescription(usize),

    #[error("El ítem '{0}' tiene cantidad inválida (debe ser > 0)")]
    ItemInvalidQuantity(usize),

    #[error("El ítem '{0}' tiene precio unitario inválido (debe ser >= 0)")]
    ItemInvalidUnitPrice(usize),

    #[error("El cliente sin RIF debe tener marcado 'sin derecho a crédito fiscal'")]
    ConsumerFinalMissingNoFiscalCreditFlag,

    #[error("La condición de pago crédito requiere indicar el plazo en días")]
    CreditConditionMissingDays,

    #[error("El plazo de crédito debe ser mayor a cero")]
    CreditConditionInvalidDays,

    #[error("La tasa de cambio debe ser mayor a cero")]
    InvalidExchangeRate,
}

/// Valida todos los datos de una factura antes de emitirla.
///
/// Verifica todos los campos obligatorios según PA 0071 del SENIAT:
/// emisor implícito (manejado por la empresa), datos del comprador, número de documento,
/// ítems, condición de pago, y reglas de consumidor final.
///
/// Retorna un `Vec` con todos los errores encontrados. Si está vacío, los datos son válidos.
pub fn validate_invoice_data(data: &InvoiceData) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Número de factura
    if data.invoice_number.trim().is_empty() {
        errors.push(ValidationError::MissingInvoiceNumber);
    }

    // Número de control
    if data.control_number.trim().is_empty() {
        errors.push(ValidationError::MissingControlNumber);
    }

    // Nombre del cliente
    if data.client_name.trim().is_empty() {
        errors.push(ValidationError::MissingClientName);
    }

    // Domicilio del cliente
    if data.client_address.trim().is_empty() {
        errors.push(ValidationError::MissingClientAddress);
    }

    // Consumidor final sin RIF debe tener bandera de sin crédito fiscal
    if data.client_rif.is_none() && !data.no_fiscal_credit {
        errors.push(ValidationError::ConsumerFinalMissingNoFiscalCreditFlag);
    }

    // Ítems
    if data.items.is_empty() {
        errors.push(ValidationError::NoItems);
    } else {
        use rust_decimal::Decimal;
        for (i, item) in data.items.iter().enumerate() {
            if item.description.trim().is_empty() {
                errors.push(ValidationError::ItemEmptyDescription(i));
            }
            if item.quantity <= Decimal::ZERO {
                errors.push(ValidationError::ItemInvalidQuantity(i));
            }
            if item.unit_price < Decimal::ZERO {
                errors.push(ValidationError::ItemInvalidUnitPrice(i));
            }
        }
    }

    // Condición de pago crédito
    if let super::builder::PaymentCondition::Credit { days } = &data.payment_condition {
        match days {
            None => errors.push(ValidationError::CreditConditionMissingDays),
            Some(0) => errors.push(ValidationError::CreditConditionInvalidDays),
            _ => {}
        }
    }

    // Tasa de cambio
    if data.exchange_rate <= rust_decimal::Decimal::ZERO {
        errors.push(ValidationError::InvalidExchangeRate);
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::invoice::builder::{InvoiceData, InvoiceItemData, PaymentCondition};
    use crate::domain::tax::iva::IvaRate;
    use rust_decimal_macros::dec;

    fn valid_invoice() -> InvoiceData {
        InvoiceData {
            invoice_number: "00000001".to_string(),
            control_number: "00-00000001".to_string(),
            invoice_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            client_rif: Some("J-12345678-9".to_string()),
            client_name: "Empresa Prueba C.A.".to_string(),
            client_address: "Av. Principal, Caracas".to_string(),
            items: vec![InvoiceItemData {
                description: "Servicio de consultoría".to_string(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                tax_rate: IvaRate::General,
            }],
            payment_condition: PaymentCondition::Cash,
            no_fiscal_credit: false,
            currency: "USD".to_string(),
            exchange_rate: dec!(36.50),
        }
    }

    #[test]
    fn test_valid_invoice_no_errors() {
        let invoice = valid_invoice();
        let errors = validate_invoice_data(&invoice);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_missing_invoice_number() {
        let mut invoice = valid_invoice();
        invoice.invoice_number = "".to_string();
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::MissingInvoiceNumber));
    }

    #[test]
    fn test_missing_control_number() {
        let mut invoice = valid_invoice();
        invoice.control_number = "".to_string();
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::MissingControlNumber));
    }

    #[test]
    fn test_missing_client_name() {
        let mut invoice = valid_invoice();
        invoice.client_name = "".to_string();
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::MissingClientName));
    }

    #[test]
    fn test_missing_client_address() {
        let mut invoice = valid_invoice();
        invoice.client_address = "".to_string();
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::MissingClientAddress));
    }

    #[test]
    fn test_consumer_final_without_no_fiscal_credit_flag() {
        let mut invoice = valid_invoice();
        invoice.client_rif = None;
        invoice.no_fiscal_credit = false;
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::ConsumerFinalMissingNoFiscalCreditFlag));
    }

    #[test]
    fn test_consumer_final_with_no_fiscal_credit_flag_ok() {
        let mut invoice = valid_invoice();
        invoice.client_rif = None;
        invoice.no_fiscal_credit = true;
        let errors = validate_invoice_data(&invoice);
        assert!(!errors.contains(&ValidationError::ConsumerFinalMissingNoFiscalCreditFlag));
    }

    #[test]
    fn test_no_items_error() {
        let mut invoice = valid_invoice();
        invoice.items = vec![];
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::NoItems));
    }

    #[test]
    fn test_item_empty_description() {
        let mut invoice = valid_invoice();
        invoice.items[0].description = "".to_string();
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::ItemEmptyDescription(0)));
    }

    #[test]
    fn test_item_zero_quantity() {
        let mut invoice = valid_invoice();
        invoice.items[0].quantity = dec!(0);
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::ItemInvalidQuantity(0)));
    }

    #[test]
    fn test_item_negative_unit_price() {
        let mut invoice = valid_invoice();
        invoice.items[0].unit_price = dec!(-1);
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::ItemInvalidUnitPrice(0)));
    }

    #[test]
    fn test_credit_condition_missing_days() {
        let mut invoice = valid_invoice();
        invoice.payment_condition = PaymentCondition::Credit { days: None };
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::CreditConditionMissingDays));
    }

    #[test]
    fn test_credit_condition_zero_days() {
        let mut invoice = valid_invoice();
        invoice.payment_condition = PaymentCondition::Credit { days: Some(0) };
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::CreditConditionInvalidDays));
    }

    #[test]
    fn test_credit_condition_valid_days_ok() {
        let mut invoice = valid_invoice();
        invoice.payment_condition = PaymentCondition::Credit { days: Some(30) };
        let errors = validate_invoice_data(&invoice);
        assert!(!errors.contains(&ValidationError::CreditConditionMissingDays));
        assert!(!errors.contains(&ValidationError::CreditConditionInvalidDays));
    }

    #[test]
    fn test_invalid_exchange_rate() {
        let mut invoice = valid_invoice();
        invoice.exchange_rate = dec!(0);
        let errors = validate_invoice_data(&invoice);
        assert!(errors.contains(&ValidationError::InvalidExchangeRate));
    }

    #[test]
    fn test_multiple_errors_collected() {
        let mut invoice = valid_invoice();
        invoice.invoice_number = "".to_string();
        invoice.client_name = "".to_string();
        invoice.items = vec![];
        let errors = validate_invoice_data(&invoice);
        assert!(errors.len() >= 3);
    }
}
