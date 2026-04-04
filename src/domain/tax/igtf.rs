//! Cálculo del IGTF — Impuesto a las Grandes Transacciones Financieras.
//!
//! Ley del IGTF + Decreto 4.972 (julio 2024):
//! - Tasa: 3% sobre pagos en divisas (USD, EUR, etc.) o criptomonedas (excepto Petro)
//! - Solo aplica si el vendedor es **Sujeto Pasivo Especial (SPE)** designado por el SENIAT
//! - Base de cálculo: monto total facturado en divisas **incluyendo el IVA**
//! - No forma parte de la base imponible del IVA (no se le calcula IVA al IGTF)
//!
//! Ejemplo:
//! ```
//! Subtotal:            $1,000.00
//! IVA (16%):             $160.00
//! Total factura:       $1,160.00
//! IGTF (3%):              $34.80
//! Total a pagar:       $1,194.80
//! ```

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

/// Tasa IGTF vigente: 3% (Decreto 4.972, julio 2024).
pub const IGTF_RATE: Decimal = dec!(0.03);

/// Monedas que activan el IGTF cuando el vendedor es SPE.
/// El bolívar (VES) nunca activa IGTF.
pub fn is_forex_currency(currency: &str) -> bool {
    !matches!(currency.to_uppercase().as_str(), "VES" | "BS" | "BSF" | "VEF")
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IgtfError {
    #[error("La base imponible IGTF no puede ser negativa")]
    NegativeBase,
}

/// Resultado del cálculo IGTF.
#[derive(Debug, Clone)]
pub struct IgtfResult {
    /// Monto base sobre el cual se calcula el IGTF (= grand_total de la factura en divisas).
    pub base: Decimal,
    /// Monto IGTF = base * 3%
    pub igtf_amount: Decimal,
    /// Total final a cobrar = base + igtf_amount
    pub total_to_pay: Decimal,
}

/// Calcula el IGTF sobre el monto total de la factura (incluyendo IVA).
///
/// Solo debe llamarse cuando:
/// - `es_contribuyente_especial == true` (la empresa es SPE)
/// - `is_forex_currency(currency) == true` (el pago es en divisas)
pub fn calculate_igtf(grand_total: Decimal) -> Result<IgtfResult, IgtfError> {
    if grand_total < Decimal::ZERO {
        return Err(IgtfError::NegativeBase);
    }
    let igtf_amount = (grand_total * IGTF_RATE).round_dp(2);
    let total_to_pay = grand_total + igtf_amount;
    Ok(IgtfResult {
        base: grand_total,
        igtf_amount,
        total_to_pay,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_igtf_standard_case() {
        // 1000 subtotal + 160 IVA = 1160 total → IGTF = 34.80
        let result = calculate_igtf(dec!(1160.00)).unwrap();
        assert_eq!(result.igtf_amount, dec!(34.80));
        assert_eq!(result.total_to_pay, dec!(1194.80));
    }

    #[test]
    fn test_igtf_zero() {
        let result = calculate_igtf(dec!(0)).unwrap();
        assert_eq!(result.igtf_amount, dec!(0.00));
        assert_eq!(result.total_to_pay, dec!(0.00));
    }

    #[test]
    fn test_igtf_negative_base_error() {
        assert!(calculate_igtf(dec!(-100)).is_err());
    }

    #[test]
    fn test_igtf_rounding() {
        // 100.01 * 3% = 3.0003 → debe redondear a 3.00
        let result = calculate_igtf(dec!(100.01)).unwrap();
        assert_eq!(result.igtf_amount, dec!(3.00));
    }

    #[test]
    fn test_is_forex_usd() {
        assert!(is_forex_currency("USD"));
        assert!(is_forex_currency("EUR"));
        assert!(is_forex_currency("usd"));
    }

    #[test]
    fn test_is_not_forex_ves() {
        assert!(!is_forex_currency("VES"));
        assert!(!is_forex_currency("BS"));
        assert!(!is_forex_currency("ves"));
    }
}
