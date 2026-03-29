//! Cálculos de retenciones de IVA conforme a la Providencia SNAT/2015/0049.
//!
//! Los contribuyentes especiales actúan como agentes de retención y deben
//! retener el 75% o 100% del IVA facturado por sus proveedores.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

/// Errores posibles en los cálculos de retención de IVA.
#[derive(Debug, Error)]
pub enum IvaWithholdingError {
    #[error("El monto de IVA no puede ser negativo: {0}")]
    NegativeIvaAmount(Decimal),
}

/// Porcentaje de retención de IVA aplicable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IvaWithholdingRate {
    /// Retención estándar: 75% del IVA facturado.
    /// Aplica a la mayoría de las operaciones entre contribuyentes especiales.
    Standard,
    /// Retención total: 100% del IVA facturado.
    /// Aplica cuando el proveedor no presenta RIF, entre otros casos.
    Full,
}

impl IvaWithholdingRate {
    /// Retorna la tasa de retención como fracción decimal (ej: 0.75 para 75%).
    pub fn rate(&self) -> Decimal {
        match self {
            IvaWithholdingRate::Standard => dec!(0.75),
            IvaWithholdingRate::Full => dec!(1.00),
        }
    }

    /// Retorna la tasa de retención como porcentaje entero.
    pub fn percentage(&self) -> Decimal {
        match self {
            IvaWithholdingRate::Standard => dec!(75),
            IvaWithholdingRate::Full => dec!(100),
        }
    }
}

/// Resultado del cálculo de retención de IVA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IvaWithholdingResult {
    /// Monto total del IVA facturado.
    pub iva_amount: Decimal,
    /// Tasa de retención aplicada como fracción decimal.
    pub withholding_rate: Decimal,
    /// Monto retenido (IVA * tasa de retención).
    pub withheld_amount: Decimal,
    /// Monto neto a pagar al proveedor (IVA - retención).
    pub net_payable: Decimal,
}

/// Calcula la retención de IVA que un contribuyente especial debe aplicar.
///
/// La retención de IVA es obligatoria para contribuyentes especiales designados
/// por el SENIAT como agentes de retención. Se retiene un porcentaje del IVA
/// facturado por el proveedor.
///
/// # Argumentos
///
/// * `iva_amount` - Monto total del IVA en la factura (debe ser >= 0).
/// * `rate` - Porcentaje de retención a aplicar (75% o 100%).
///
/// # Errores
///
/// Retorna `IvaWithholdingError::NegativeIvaAmount` si el monto de IVA es negativo.
pub fn calculate_iva_withholding(
    iva_amount: Decimal,
    rate: IvaWithholdingRate,
) -> Result<IvaWithholdingResult, IvaWithholdingError> {
    if iva_amount < Decimal::ZERO {
        return Err(IvaWithholdingError::NegativeIvaAmount(iva_amount));
    }

    let withholding_rate = rate.rate();
    let withheld_amount = (iva_amount * withholding_rate).round_dp(2);
    let net_payable = (iva_amount - withheld_amount).round_dp(2);

    Ok(IvaWithholdingResult {
        iva_amount,
        withholding_rate,
        withheld_amount,
        net_payable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_standard_withholding_75_percent() {
        let result = calculate_iva_withholding(dec!(160.00), IvaWithholdingRate::Standard).unwrap();
        assert_eq!(result.withholding_rate, dec!(0.75));
        assert_eq!(result.withheld_amount, dec!(120.00));
        assert_eq!(result.net_payable, dec!(40.00));
    }

    #[test]
    fn test_full_withholding_100_percent() {
        let result = calculate_iva_withholding(dec!(160.00), IvaWithholdingRate::Full).unwrap();
        assert_eq!(result.withholding_rate, dec!(1.00));
        assert_eq!(result.withheld_amount, dec!(160.00));
        assert_eq!(result.net_payable, dec!(0.00));
    }

    #[test]
    fn test_zero_iva_amount() {
        let result = calculate_iva_withholding(dec!(0.00), IvaWithholdingRate::Standard).unwrap();
        assert_eq!(result.withheld_amount, dec!(0.00));
        assert_eq!(result.net_payable, dec!(0.00));
    }

    #[test]
    fn test_negative_iva_error() {
        let result = calculate_iva_withholding(dec!(-10.00), IvaWithholdingRate::Standard);
        assert!(result.is_err());
    }

    #[test]
    fn test_small_iva_amount_standard() {
        // 0.10 * 0.75 = 0.075 -> rounds to 0.08
        let result = calculate_iva_withholding(dec!(0.10), IvaWithholdingRate::Standard).unwrap();
        assert_eq!(result.withheld_amount, dec!(0.08));
        assert_eq!(result.net_payable, dec!(0.02));
    }

    #[test]
    fn test_small_iva_amount_full() {
        let result = calculate_iva_withholding(dec!(0.01), IvaWithholdingRate::Full).unwrap();
        assert_eq!(result.withheld_amount, dec!(0.01));
        assert_eq!(result.net_payable, dec!(0.00));
    }

    #[test]
    fn test_large_iva_amount() {
        let result =
            calculate_iva_withholding(dec!(1000000.00), IvaWithholdingRate::Standard).unwrap();
        assert_eq!(result.withheld_amount, dec!(750000.00));
        assert_eq!(result.net_payable, dec!(250000.00));
    }

    #[test]
    fn test_rounding_standard() {
        // 33.33 * 0.75 = 24.9975 -> rounds to 25.00
        let result = calculate_iva_withholding(dec!(33.33), IvaWithholdingRate::Standard).unwrap();
        assert_eq!(result.withheld_amount, dec!(25.00));
        assert_eq!(result.net_payable, dec!(8.33));
    }

    #[test]
    fn test_rate_percentage_display() {
        assert_eq!(IvaWithholdingRate::Standard.percentage(), dec!(75));
        assert_eq!(IvaWithholdingRate::Full.percentage(), dec!(100));
    }

    #[test]
    fn test_withheld_plus_net_equals_iva() {
        let iva = dec!(12345.67);
        let result = calculate_iva_withholding(iva, IvaWithholdingRate::Standard).unwrap();
        assert_eq!(result.withheld_amount + result.net_payable, iva);
    }

    #[test]
    fn test_full_withheld_plus_net_equals_iva() {
        let iva = dec!(9999.99);
        let result = calculate_iva_withholding(iva, IvaWithholdingRate::Full).unwrap();
        assert_eq!(result.withheld_amount + result.net_payable, iva);
    }
}
