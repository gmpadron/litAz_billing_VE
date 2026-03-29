//! Entradas del Libro de Compras (obligatorio, mensual, SENIAT).
//!
//! Cada entrada representa una compra registrada con todos los campos
//! requeridos por el SENIAT: proveedor, RIF, montos desglosados por
//! alicuota, retenciones aplicadas, etc.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::BookError;

/// Representa una entrada en el Libro de Compras fiscal.
///
/// Contiene todos los campos obligatorios segun la normativa SENIAT
/// para el registro de compras mensuales.
#[derive(Debug, Clone)]
pub struct PurchaseBookEntry {
    /// Fecha de la operacion de compra.
    pub entry_date: NaiveDate,
    /// Nombre o razon social del proveedor.
    pub supplier_name: String,
    /// RIF del proveedor (formato validado).
    pub supplier_rif: String,
    /// Numero de factura del proveedor.
    pub invoice_number: String,
    /// Numero de control de la factura del proveedor.
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
    /// Monto de IVA retenido al proveedor (si aplica).
    pub iva_withheld: Option<Decimal>,
    /// Periodo fiscal en formato YYYY-MM.
    pub period: String,
}

impl PurchaseBookEntry {
    /// Valida que la entrada del libro de compras tenga todos los campos
    /// obligatorios y que los valores sean coherentes.
    ///
    /// # Validaciones
    ///
    /// - Nombre del proveedor no vacio
    /// - RIF del proveedor no vacio
    /// - Numero de factura no vacio
    /// - Numero de control no vacio
    /// - Periodo en formato YYYY-MM
    /// - Montos no negativos
    ///
    /// # Errores
    ///
    /// Retorna `BookError` si alguna validacion falla.
    pub fn validate(&self) -> Result<(), BookError> {
        if self.supplier_name.trim().is_empty() {
            return Err(BookError::MissingRequiredField {
                field: "supplier_name".to_string(),
            });
        }

        if self.supplier_rif.trim().is_empty() {
            return Err(BookError::MissingRequiredField {
                field: "supplier_rif".to_string(),
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

        Ok(())
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

        if let Some(withheld) = self.iva_withheld
            && withheld < Decimal::ZERO
        {
            return Err(BookError::NegativeAmount {
                field: "iva_withheld".to_string(),
                value: withheld.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn valid_entry() -> PurchaseBookEntry {
        PurchaseBookEntry {
            entry_date: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            supplier_name: "Proveedor ABC, C.A.".to_string(),
            supplier_rif: "J-12345678-9".to_string(),
            invoice_number: "00000042".to_string(),
            control_number: "00-00001234".to_string(),
            total_amount: dec!(11600.00),
            exempt_base: dec!(0.00),
            general_base: dec!(10000.00),
            general_tax: dec!(1600.00),
            reduced_base: dec!(0.00),
            reduced_tax: dec!(0.00),
            luxury_base: dec!(0.00),
            luxury_tax: dec!(0.00),
            iva_withheld: Some(dec!(1200.00)),
            period: "2026-03".to_string(),
        }
    }

    #[test]
    fn test_valid_entry_passes_validation() {
        let entry = valid_entry();
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_missing_supplier_name() {
        let mut entry = valid_entry();
        entry.supplier_name = "".to_string();
        let result = entry.validate();
        assert!(matches!(
            result,
            Err(BookError::MissingRequiredField { field }) if field == "supplier_name"
        ));
    }

    #[test]
    fn test_whitespace_supplier_name() {
        let mut entry = valid_entry();
        entry.supplier_name = "   ".to_string();
        assert!(entry.validate().is_err());
    }

    #[test]
    fn test_missing_supplier_rif() {
        let mut entry = valid_entry();
        entry.supplier_rif = "".to_string();
        assert!(matches!(
            entry.validate(),
            Err(BookError::MissingRequiredField { field }) if field == "supplier_rif"
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
    fn test_invalid_period_format() {
        let mut entry = valid_entry();
        entry.period = "2026/03".to_string();
        assert!(matches!(
            entry.validate(),
            Err(BookError::InvalidPeriodFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_period_month_13() {
        let mut entry = valid_entry();
        entry.period = "2026-13".to_string();
        assert!(matches!(
            entry.validate(),
            Err(BookError::InvalidPeriodFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_period_month_0() {
        let mut entry = valid_entry();
        entry.period = "2026-00".to_string();
        assert!(matches!(
            entry.validate(),
            Err(BookError::InvalidPeriodFormat { .. })
        ));
    }

    #[test]
    fn test_negative_total_amount() {
        let mut entry = valid_entry();
        entry.total_amount = dec!(-100.00);
        assert!(matches!(
            entry.validate(),
            Err(BookError::NegativeAmount { field, .. }) if field == "total_amount"
        ));
    }

    #[test]
    fn test_negative_general_base() {
        let mut entry = valid_entry();
        entry.general_base = dec!(-1.00);
        assert!(matches!(
            entry.validate(),
            Err(BookError::NegativeAmount { field, .. }) if field == "general_base"
        ));
    }

    #[test]
    fn test_negative_iva_withheld() {
        let mut entry = valid_entry();
        entry.iva_withheld = Some(dec!(-500.00));
        assert!(matches!(
            entry.validate(),
            Err(BookError::NegativeAmount { field, .. }) if field == "iva_withheld"
        ));
    }

    #[test]
    fn test_zero_amounts_valid() {
        let mut entry = valid_entry();
        entry.total_amount = dec!(0.00);
        entry.exempt_base = dec!(0.00);
        entry.general_base = dec!(0.00);
        entry.general_tax = dec!(0.00);
        entry.reduced_base = dec!(0.00);
        entry.reduced_tax = dec!(0.00);
        entry.iva_withheld = None;
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_no_iva_withheld_valid() {
        let mut entry = valid_entry();
        entry.iva_withheld = None;
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_exempt_only_entry() {
        let mut entry = valid_entry();
        entry.total_amount = dec!(5000.00);
        entry.exempt_base = dec!(5000.00);
        entry.general_base = dec!(0.00);
        entry.general_tax = dec!(0.00);
        entry.reduced_base = dec!(0.00);
        entry.reduced_tax = dec!(0.00);
        entry.iva_withheld = None;
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_large_amounts() {
        let mut entry = valid_entry();
        entry.total_amount = dec!(999999999999999.99);
        entry.general_base = dec!(862068965517241.37);
        entry.general_tax = dec!(137931034482758.62);
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_very_small_amounts() {
        let mut entry = valid_entry();
        entry.total_amount = dec!(0.01);
        entry.general_base = dec!(0.01);
        entry.general_tax = dec!(0.00);
        assert!(entry.validate().is_ok());
    }
}
