//! Entradas del Libro de Ventas (obligatorio, mensual, SENIAT).
//!
//! Cada entrada representa una venta registrada con todos los campos
//! requeridos por el SENIAT. Las ventas a consumidores finales sin RIF
//! se pueden agrupar en resumen diario (ver `daily_summary`).

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::BookError;

/// Representa una entrada en el Libro de Ventas fiscal.
///
/// Contiene todos los campos obligatorios segun la normativa SENIAT
/// para el registro de ventas mensuales.
#[derive(Debug, Clone)]
pub struct SalesBookEntry {
    /// Fecha de la operacion de venta.
    pub entry_date: NaiveDate,
    /// Nombre o razon social del comprador.
    pub buyer_name: String,
    /// RIF del comprador. Puede ser `None` para consumidores finales sin RIF.
    pub buyer_rif: Option<String>,
    /// Numero de factura emitida.
    pub invoice_number: String,
    /// Numero de control de la factura.
    pub control_number: String,
    /// Monto total de la factura (incluyendo IVA).
    pub total_amount: Decimal,
    /// Base imponible exenta de IVA.
    pub exempt_base: Decimal,
    /// Base imponible gravada con alicuota general (16%).
    pub general_base: Decimal,
    /// Monto del IVA general (16% sobre general_base).
    pub general_tax: Decimal,
    /// Base imponible gravada con alicuota reducida (8%).
    pub reduced_base: Decimal,
    /// Monto del IVA reducido (8% sobre reduced_base).
    pub reduced_tax: Decimal,
    /// Base imponible gravada con alicuota de lujo (31%).
    pub luxury_base: Decimal,
    /// Monto del IVA de lujo (31% sobre luxury_base).
    pub luxury_tax: Decimal,
    /// Indica si esta entrada corresponde a un resumen diario de
    /// ventas a consumidores finales sin RIF.
    pub is_summary: bool,
    /// Periodo fiscal en formato YYYY-MM.
    pub period: String,
}

impl SalesBookEntry {
    /// Valida que la entrada del libro de ventas tenga todos los campos
    /// obligatorios y que los valores sean coherentes.
    ///
    /// # Validaciones
    ///
    /// - Nombre del comprador no vacio
    /// - Numero de factura no vacio
    /// - Numero de control no vacio
    /// - Periodo en formato YYYY-MM
    /// - Montos no negativos
    /// - Si `is_summary` es true, `buyer_rif` debe ser `None`
    ///
    /// # Errores
    ///
    /// Retorna `BookError` si alguna validacion falla.
    pub fn validate(&self) -> Result<(), BookError> {
        if self.buyer_name.trim().is_empty() {
            return Err(BookError::MissingRequiredField {
                field: "buyer_name".to_string(),
            });
        }

        if self.invoice_number.trim().is_empty() {
            return Err(BookError::MissingRequiredField {
                field: "invoice_number".to_string(),
            });
        }

        if self.control_number.trim().is_empty() {
            return Err(BookError::MissingRequiredField {
                field: "control_number".to_string(),
            });
        }

        Self::validate_period(&self.period)?;

        self.validate_amounts()?;

        if self.is_summary && self.buyer_rif.is_some() {
            return Err(BookError::MissingRequiredField {
                field: "is_summary=true requires buyer_rif=None (resumen diario es solo para consumidor final)".to_string(),
            });
        }

        Ok(())
    }

    /// Indica si esta venta es a un consumidor final sin RIF.
    ///
    /// Las facturas a consumidores finales sin RIF deben llevar la leyenda
    /// "SIN DERECHO A CREDITO FISCAL" y pueden agruparse en resumen diario
    /// en el libro de ventas.
    pub fn is_consumer_final(&self) -> bool {
        self.buyer_rif.is_none()
    }

    /// Valida el formato del periodo fiscal (YYYY-MM).
    fn validate_period(period: &str) -> Result<(), BookError> {
        let parts: Vec<&str> = period.split('-').collect();
        if parts.len() != 2 {
            return Err(BookError::InvalidPeriodFormat {
                value: period.to_string(),
            });
        }

        let year: Result<u32, _> = parts[0].parse();
        let month: Result<u32, _> = parts[1].parse();

        match (year, month) {
            (Ok(y), Ok(m)) if y >= 2000 && y <= 9999 && m >= 1 && m <= 12 => Ok(()),
            _ => Err(BookError::InvalidPeriodFormat {
                value: period.to_string(),
            }),
        }
    }

    /// Valida que los montos no sean negativos.
    fn validate_amounts(&self) -> Result<(), BookError> {
        let fields = [
            ("total_amount", self.total_amount),
            ("exempt_base", self.exempt_base),
            ("general_base", self.general_base),
            ("general_tax", self.general_tax),
            ("reduced_base", self.reduced_base),
            ("reduced_tax", self.reduced_tax),
            ("luxury_base", self.luxury_base),
            ("luxury_tax", self.luxury_tax),
        ];

        for (name, value) in &fields {
            if *value < Decimal::ZERO {
                return Err(BookError::NegativeAmount {
                    field: name.to_string(),
                    value: value.to_string(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn valid_entry() -> SalesBookEntry {
        SalesBookEntry {
            entry_date: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            buyer_name: "Cliente XYZ, C.A.".to_string(),
            buyer_rif: Some("J-98765432-1".to_string()),
            invoice_number: "00000001".to_string(),
            control_number: "00-00000001".to_string(),
            total_amount: dec!(23200.00),
            exempt_base: dec!(0.00),
            general_base: dec!(20000.00),
            general_tax: dec!(3200.00),
            reduced_base: dec!(0.00),
            reduced_tax: dec!(0.00),
            luxury_base: dec!(0.00),
            luxury_tax: dec!(0.00),
            is_summary: false,
            period: "2026-03".to_string(),
        }
    }

    fn consumer_final_entry() -> SalesBookEntry {
        SalesBookEntry {
            entry_date: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            buyer_name: "Consumidor Final".to_string(),
            buyer_rif: None,
            invoice_number: "00000002".to_string(),
            control_number: "00-00000002".to_string(),
            total_amount: dec!(580.00),
            exempt_base: dec!(0.00),
            general_base: dec!(500.00),
            general_tax: dec!(80.00),
            reduced_base: dec!(0.00),
            reduced_tax: dec!(0.00),
            luxury_base: dec!(0.00),
            luxury_tax: dec!(0.00),
            is_summary: false,
            period: "2026-03".to_string(),
        }
    }

    #[test]
    fn test_valid_entry_passes_validation() {
        assert!(valid_entry().validate().is_ok());
    }

    #[test]
    fn test_consumer_final_entry_valid() {
        assert!(consumer_final_entry().validate().is_ok());
    }

    #[test]
    fn test_is_consumer_final_with_rif() {
        let entry = valid_entry();
        assert!(!entry.is_consumer_final());
    }

    #[test]
    fn test_is_consumer_final_without_rif() {
        let entry = consumer_final_entry();
        assert!(entry.is_consumer_final());
    }

    #[test]
    fn test_missing_buyer_name() {
        let mut entry = valid_entry();
        entry.buyer_name = "".to_string();
        assert!(matches!(
            entry.validate(),
            Err(BookError::MissingRequiredField { field }) if field == "buyer_name"
        ));
    }

    #[test]
    fn test_missing_invoice_number() {
        let mut entry = valid_entry();
        entry.invoice_number = "".to_string();
        assert!(matches!(
            entry.validate(),
            Err(BookError::MissingRequiredField { field }) if field == "invoice_number"
        ));
    }

    #[test]
    fn test_missing_control_number() {
        let mut entry = valid_entry();
        entry.control_number = "".to_string();
        assert!(matches!(
            entry.validate(),
            Err(BookError::MissingRequiredField { field }) if field == "control_number"
        ));
    }

    #[test]
    fn test_invalid_period() {
        let mut entry = valid_entry();
        entry.period = "marzo-2026".to_string();
        assert!(matches!(
            entry.validate(),
            Err(BookError::InvalidPeriodFormat { .. })
        ));
    }

    #[test]
    fn test_negative_amount() {
        let mut entry = valid_entry();
        entry.total_amount = dec!(-1.00);
        assert!(matches!(
            entry.validate(),
            Err(BookError::NegativeAmount { .. })
        ));
    }

    #[test]
    fn test_summary_entry() {
        let mut entry = consumer_final_entry();
        entry.is_summary = true;
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_zero_amounts_valid() {
        let mut entry = valid_entry();
        entry.total_amount = dec!(0.00);
        entry.general_base = dec!(0.00);
        entry.general_tax = dec!(0.00);
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_exempt_only_sale() {
        let mut entry = valid_entry();
        entry.total_amount = dec!(1000.00);
        entry.exempt_base = dec!(1000.00);
        entry.general_base = dec!(0.00);
        entry.general_tax = dec!(0.00);
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_reduced_rate_sale() {
        let mut entry = valid_entry();
        entry.total_amount = dec!(10800.00);
        entry.general_base = dec!(0.00);
        entry.general_tax = dec!(0.00);
        entry.reduced_base = dec!(10000.00);
        entry.reduced_tax = dec!(800.00);
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_large_amounts() {
        let mut entry = valid_entry();
        entry.total_amount = dec!(999999999999.99);
        entry.general_base = dec!(862068965517.24);
        entry.general_tax = dec!(137931034482.75);
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_negative_luxury_base() {
        let mut entry = valid_entry();
        entry.luxury_base = dec!(-1.00);
        assert!(matches!(
            entry.validate(),
            Err(BookError::NegativeAmount { field, .. }) if field == "luxury_base"
        ));
    }

    #[test]
    fn test_negative_luxury_tax() {
        let mut entry = valid_entry();
        entry.luxury_tax = dec!(-1.00);
        assert!(matches!(
            entry.validate(),
            Err(BookError::NegativeAmount { field, .. }) if field == "luxury_tax"
        ));
    }

    #[test]
    fn test_summary_with_rif_fails() {
        let mut entry = valid_entry();
        entry.is_summary = true;
        entry.buyer_rif = Some("J-12345678-9".to_string());
        assert!(entry.validate().is_err());
    }

    #[test]
    fn test_summary_without_rif_passes() {
        let mut entry = consumer_final_entry();
        entry.is_summary = true;
        assert!(entry.validate().is_ok());
    }
}
