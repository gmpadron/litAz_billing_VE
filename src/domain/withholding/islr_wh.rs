//! Cálculos de retenciones de ISLR conforme al Decreto 1.808.
//!
//! Las retenciones de ISLR se declaran mensualmente ante el SENIAT.
//! Algunos conceptos aplican un "sustraendo" (subtract_amount) que reduce
//! el monto de la retención cuando el pago supera cierto umbral.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

/// Errores posibles en los cálculos de retención de ISLR.
#[derive(Debug, Error)]
pub enum IslrWithholdingError {
    #[error("El monto base no puede ser negativo: {0}")]
    NegativeBaseAmount(Decimal),
    #[error("La tasa de retención debe estar entre 0 y 1 (0%-100%): {0}")]
    InvalidRate(Decimal),
    #[error("El sustraendo no puede ser negativo: {0}")]
    NegativeSubtractAmount(Decimal),
}

/// Resultado del cálculo de retención de ISLR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IslrWithholdingResult {
    /// Monto base sobre el cual se calcula la retención.
    pub base_amount: Decimal,
    /// Tasa de retención aplicada como fracción decimal.
    pub rate: Decimal,
    /// Sustraendo aplicado (puede ser cero).
    pub subtract_amount: Decimal,
    /// Monto retenido (base * tasa - sustraendo, mínimo 0).
    pub withheld_amount: Decimal,
    /// Monto neto a pagar (base - retención).
    pub net_payable: Decimal,
}

/// Calcula la retención de ISLR sobre un pago.
///
/// Fórmula: retención = (base * tasa) - sustraendo
/// Si el resultado es negativo, la retención es cero.
///
/// # Argumentos
///
/// * `base` - Monto base del pago o abono en cuenta (debe ser >= 0).
/// * `rate` - Tasa de retención como fracción decimal (ej: 0.05 para 5%). Debe estar entre 0 y 1.
///
/// # Errores
///
/// - `IslrWithholdingError::NegativeBaseAmount` si el monto base es negativo.
/// - `IslrWithholdingError::InvalidRate` si la tasa no está entre 0 y 1.
pub fn calculate_islr_withholding(
    base: Decimal,
    rate: Decimal,
) -> Result<IslrWithholdingResult, IslrWithholdingError> {
    calculate_islr_withholding_with_subtract(base, rate, Decimal::ZERO)
}

/// Calcula la retención de ISLR con sustraendo.
///
/// El sustraendo es un monto fijo que se resta de la retención bruta.
/// Se usa en ciertos tramos del ISLR donde la ley establece un monto
/// a deducir para evitar doble tributación en escalas progresivas.
///
/// # Argumentos
///
/// * `base` - Monto base del pago o abono en cuenta (debe ser >= 0).
/// * `rate` - Tasa de retención como fracción decimal (ej: 0.05 para 5%).
/// * `subtract_amount` - Sustraendo a aplicar (debe ser >= 0).
///
/// # Errores
///
/// - `IslrWithholdingError::NegativeBaseAmount` si el monto base es negativo.
/// - `IslrWithholdingError::InvalidRate` si la tasa no está entre 0 y 1.
/// - `IslrWithholdingError::NegativeSubtractAmount` si el sustraendo es negativo.
pub fn calculate_islr_withholding_with_subtract(
    base: Decimal,
    rate: Decimal,
    subtract_amount: Decimal,
) -> Result<IslrWithholdingResult, IslrWithholdingError> {
    if base < Decimal::ZERO {
        return Err(IslrWithholdingError::NegativeBaseAmount(base));
    }
    if rate < Decimal::ZERO || rate > dec!(1.00) {
        return Err(IslrWithholdingError::InvalidRate(rate));
    }
    if subtract_amount < Decimal::ZERO {
        return Err(IslrWithholdingError::NegativeSubtractAmount(
            subtract_amount,
        ));
    }

    let gross_withholding = base * rate;
    let withheld_amount = if gross_withholding > subtract_amount {
        (gross_withholding - subtract_amount).round_dp(2)
    } else {
        Decimal::ZERO
    };
    let net_payable = (base - withheld_amount).round_dp(2);

    Ok(IslrWithholdingResult {
        base_amount: base,
        rate,
        subtract_amount,
        withheld_amount,
        net_payable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // --- calculate_islr_withholding (sin sustraendo) ---

    #[test]
    fn test_basic_5_percent() {
        let result = calculate_islr_withholding(dec!(10000.00), dec!(0.05)).unwrap();
        assert_eq!(result.rate, dec!(0.05));
        assert_eq!(result.subtract_amount, dec!(0));
        assert_eq!(result.withheld_amount, dec!(500.00));
        assert_eq!(result.net_payable, dec!(9500.00));
    }

    #[test]
    fn test_basic_3_percent() {
        let result = calculate_islr_withholding(dec!(5000.00), dec!(0.03)).unwrap();
        assert_eq!(result.withheld_amount, dec!(150.00));
        assert_eq!(result.net_payable, dec!(4850.00));
    }

    #[test]
    fn test_basic_1_percent() {
        let result = calculate_islr_withholding(dec!(100000.00), dec!(0.01)).unwrap();
        assert_eq!(result.withheld_amount, dec!(1000.00));
        assert_eq!(result.net_payable, dec!(99000.00));
    }

    #[test]
    fn test_basic_34_percent_non_resident() {
        let result = calculate_islr_withholding(dec!(50000.00), dec!(0.34)).unwrap();
        assert_eq!(result.withheld_amount, dec!(17000.00));
        assert_eq!(result.net_payable, dec!(33000.00));
    }

    #[test]
    fn test_zero_base() {
        let result = calculate_islr_withholding(dec!(0.00), dec!(0.05)).unwrap();
        assert_eq!(result.withheld_amount, dec!(0.00));
        assert_eq!(result.net_payable, dec!(0.00));
    }

    #[test]
    fn test_zero_rate() {
        let result = calculate_islr_withholding(dec!(1000.00), dec!(0.00)).unwrap();
        assert_eq!(result.withheld_amount, dec!(0.00));
        assert_eq!(result.net_payable, dec!(1000.00));
    }

    #[test]
    fn test_full_rate_100_percent() {
        let result = calculate_islr_withholding(dec!(1000.00), dec!(1.00)).unwrap();
        assert_eq!(result.withheld_amount, dec!(1000.00));
        assert_eq!(result.net_payable, dec!(0.00));
    }

    #[test]
    fn test_negative_base_error() {
        let result = calculate_islr_withholding(dec!(-100.00), dec!(0.05));
        assert!(result.is_err());
    }

    #[test]
    fn test_negative_rate_error() {
        let result = calculate_islr_withholding(dec!(1000.00), dec!(-0.05));
        assert!(result.is_err());
    }

    #[test]
    fn test_rate_above_100_error() {
        let result = calculate_islr_withholding(dec!(1000.00), dec!(1.50));
        assert!(result.is_err());
    }

    #[test]
    fn test_rounding() {
        // 333.33 * 0.05 = 16.6665 -> rounds to 16.67
        let result = calculate_islr_withholding(dec!(333.33), dec!(0.05)).unwrap();
        assert_eq!(result.withheld_amount, dec!(16.67));
        assert_eq!(result.net_payable, dec!(316.66));
    }

    #[test]
    fn test_very_small_amount() {
        // 0.01 * 0.01 = 0.0001 -> rounds to 0.00
        let result = calculate_islr_withholding(dec!(0.01), dec!(0.01)).unwrap();
        assert_eq!(result.withheld_amount, dec!(0.00));
        assert_eq!(result.net_payable, dec!(0.01));
    }

    #[test]
    fn test_very_large_amount() {
        let result = calculate_islr_withholding(dec!(999999999.99), dec!(0.34)).unwrap();
        assert_eq!(result.withheld_amount, dec!(340000000.00));
        assert_eq!(result.net_payable, dec!(659999999.99));
    }

    // --- calculate_islr_withholding_with_subtract ---

    #[test]
    fn test_with_subtract_amount() {
        // 10000 * 0.05 = 500 - 100 (sustraendo) = 400
        let result =
            calculate_islr_withholding_with_subtract(dec!(10000.00), dec!(0.05), dec!(100.00))
                .unwrap();
        assert_eq!(result.subtract_amount, dec!(100.00));
        assert_eq!(result.withheld_amount, dec!(400.00));
        assert_eq!(result.net_payable, dec!(9600.00));
    }

    #[test]
    fn test_subtract_exceeds_gross_withholding() {
        // 100 * 0.05 = 5 - 10 (sustraendo) -> retención = 0 (no puede ser negativa)
        let result =
            calculate_islr_withholding_with_subtract(dec!(100.00), dec!(0.05), dec!(10.00))
                .unwrap();
        assert_eq!(result.withheld_amount, dec!(0.00));
        assert_eq!(result.net_payable, dec!(100.00));
    }

    #[test]
    fn test_subtract_equals_gross_withholding() {
        // 1000 * 0.05 = 50 - 50 = 0
        let result =
            calculate_islr_withholding_with_subtract(dec!(1000.00), dec!(0.05), dec!(50.00))
                .unwrap();
        assert_eq!(result.withheld_amount, dec!(0.00));
        assert_eq!(result.net_payable, dec!(1000.00));
    }

    #[test]
    fn test_negative_subtract_error() {
        let result =
            calculate_islr_withholding_with_subtract(dec!(1000.00), dec!(0.05), dec!(-10.00));
        assert!(result.is_err());
    }

    #[test]
    fn test_withheld_plus_net_equals_base() {
        let base = dec!(12345.67);
        let result = calculate_islr_withholding(base, dec!(0.05)).unwrap();
        assert_eq!(result.withheld_amount + result.net_payable, base);
    }

    #[test]
    fn test_with_subtract_withheld_plus_net_equals_base() {
        let base = dec!(50000.00);
        let result =
            calculate_islr_withholding_with_subtract(base, dec!(0.34), dec!(1000.00)).unwrap();
        assert_eq!(result.withheld_amount + result.net_payable, base);
    }
}
