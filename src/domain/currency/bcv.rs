//! Conversión de moneda entre Bolívares (VES) y Dólares (USD) usando la tasa BCV.
//!
//! El Banco Central de Venezuela (BCV) publica diariamente la tasa oficial
//! de cambio. Toda factura en Venezuela debe expresar los montos tanto en
//! Bolívares como en USD, usando la tasa BCV del día de la operación.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;

/// Errores posibles en las conversiones de moneda.
#[derive(Debug, Error)]
pub enum CurrencyError {
    #[error("El monto no puede ser negativo: {0}")]
    NegativeAmount(Decimal),
    #[error("La tasa de cambio debe ser mayor que cero: {0}")]
    InvalidExchangeRate(Decimal),
}

/// Monedas soportadas por el sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Currency {
    /// Bolívares (moneda de curso legal en Venezuela).
    VES,
    /// Dólares estadounidenses.
    USD,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::VES => write!(f, "VES"),
            Currency::USD => write!(f, "USD"),
        }
    }
}

/// Resultado de una conversión de moneda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeResult {
    /// Monto original antes de la conversión.
    pub original_amount: Decimal,
    /// Moneda del monto original.
    pub original_currency: Currency,
    /// Monto convertido.
    pub converted_amount: Decimal,
    /// Moneda destino.
    pub target_currency: Currency,
    /// Tasa de cambio BCV utilizada (VES por 1 USD).
    pub exchange_rate: Decimal,
    /// Fecha de la tasa de cambio utilizada.
    pub rate_date: NaiveDate,
}

/// Convierte un monto de USD a VES usando la tasa BCV del día.
///
/// La tasa BCV representa cuántos Bolívares equivale 1 USD.
/// Fórmula: VES = USD * tasa_bcv
///
/// # Argumentos
///
/// * `amount` - Monto en USD a convertir (debe ser >= 0).
/// * `bcv_rate` - Tasa BCV del día (VES por 1 USD, debe ser > 0).
/// * `rate_date` - Fecha de la tasa utilizada.
///
/// # Errores
///
/// - `CurrencyError::NegativeAmount` si el monto es negativo.
/// - `CurrencyError::InvalidExchangeRate` si la tasa es <= 0.
pub fn convert_to_ves(
    amount: Decimal,
    bcv_rate: Decimal,
    rate_date: NaiveDate,
) -> Result<ExchangeResult, CurrencyError> {
    validate_inputs(amount, bcv_rate)?;

    let converted = (amount * bcv_rate).round_dp(2);

    Ok(ExchangeResult {
        original_amount: amount,
        original_currency: Currency::USD,
        converted_amount: converted,
        target_currency: Currency::VES,
        exchange_rate: bcv_rate,
        rate_date,
    })
}

/// Convierte un monto de VES a USD usando la tasa BCV del día.
///
/// La tasa BCV representa cuántos Bolívares equivale 1 USD.
/// Fórmula: USD = VES / tasa_bcv
///
/// # Argumentos
///
/// * `amount` - Monto en VES a convertir (debe ser >= 0).
/// * `bcv_rate` - Tasa BCV del día (VES por 1 USD, debe ser > 0).
/// * `rate_date` - Fecha de la tasa utilizada.
///
/// # Errores
///
/// - `CurrencyError::NegativeAmount` si el monto es negativo.
/// - `CurrencyError::InvalidExchangeRate` si la tasa es <= 0.
pub fn convert_to_usd(
    amount: Decimal,
    bcv_rate: Decimal,
    rate_date: NaiveDate,
) -> Result<ExchangeResult, CurrencyError> {
    validate_inputs(amount, bcv_rate)?;

    let converted = (amount / bcv_rate).round_dp(2);

    Ok(ExchangeResult {
        original_amount: amount,
        original_currency: Currency::VES,
        converted_amount: converted,
        target_currency: Currency::USD,
        exchange_rate: bcv_rate,
        rate_date,
    })
}

/// Valida que el monto y la tasa de cambio sean válidos.
fn validate_inputs(amount: Decimal, bcv_rate: Decimal) -> Result<(), CurrencyError> {
    if amount < Decimal::ZERO {
        return Err(CurrencyError::NegativeAmount(amount));
    }
    if bcv_rate <= Decimal::ZERO {
        return Err(CurrencyError::InvalidExchangeRate(bcv_rate));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn test_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, 26).unwrap()
    }

    // --- convert_to_ves tests ---

    #[test]
    fn test_usd_to_ves_basic() {
        let result = convert_to_ves(dec!(100.00), dec!(36.50), test_date()).unwrap();
        assert_eq!(result.original_amount, dec!(100.00));
        assert_eq!(result.original_currency, Currency::USD);
        assert_eq!(result.converted_amount, dec!(3650.00));
        assert_eq!(result.target_currency, Currency::VES);
        assert_eq!(result.exchange_rate, dec!(36.50));
        assert_eq!(result.rate_date, test_date());
    }

    #[test]
    fn test_usd_to_ves_with_decimals() {
        // 55.75 * 36.50 = 2034.875 -> rounds to 2034.88
        let result = convert_to_ves(dec!(55.75), dec!(36.50), test_date()).unwrap();
        assert_eq!(result.converted_amount, dec!(2034.88));
    }

    #[test]
    fn test_usd_to_ves_zero_amount() {
        let result = convert_to_ves(dec!(0.00), dec!(36.50), test_date()).unwrap();
        assert_eq!(result.converted_amount, dec!(0.00));
    }

    #[test]
    fn test_usd_to_ves_rate_of_one() {
        let result = convert_to_ves(dec!(100.00), dec!(1.00), test_date()).unwrap();
        assert_eq!(result.converted_amount, dec!(100.00));
    }

    #[test]
    fn test_usd_to_ves_very_high_rate() {
        // Simulating historically high Venezuelan exchange rates
        let result = convert_to_ves(dec!(100.00), dec!(4500000.00), test_date()).unwrap();
        assert_eq!(result.converted_amount, dec!(450000000.00));
    }

    #[test]
    fn test_usd_to_ves_small_amount() {
        // 0.01 * 36.50 = 0.365 -> rounds to 0.36 (banker's rounding)
        let result = convert_to_ves(dec!(0.01), dec!(36.50), test_date()).unwrap();
        assert_eq!(result.converted_amount, dec!(0.36));
    }

    #[test]
    fn test_usd_to_ves_negative_amount_error() {
        let result = convert_to_ves(dec!(-10.00), dec!(36.50), test_date());
        assert!(result.is_err());
    }

    #[test]
    fn test_usd_to_ves_zero_rate_error() {
        let result = convert_to_ves(dec!(100.00), dec!(0.00), test_date());
        assert!(result.is_err());
    }

    #[test]
    fn test_usd_to_ves_negative_rate_error() {
        let result = convert_to_ves(dec!(100.00), dec!(-36.50), test_date());
        assert!(result.is_err());
    }

    // --- convert_to_usd tests ---

    #[test]
    fn test_ves_to_usd_basic() {
        let result = convert_to_usd(dec!(3650.00), dec!(36.50), test_date()).unwrap();
        assert_eq!(result.original_amount, dec!(3650.00));
        assert_eq!(result.original_currency, Currency::VES);
        assert_eq!(result.converted_amount, dec!(100.00));
        assert_eq!(result.target_currency, Currency::USD);
    }

    #[test]
    fn test_ves_to_usd_with_rounding() {
        // 1000.00 / 36.50 = 27.397260... -> rounds to 27.40
        let result = convert_to_usd(dec!(1000.00), dec!(36.50), test_date()).unwrap();
        assert_eq!(result.converted_amount, dec!(27.40));
    }

    #[test]
    fn test_ves_to_usd_zero_amount() {
        let result = convert_to_usd(dec!(0.00), dec!(36.50), test_date()).unwrap();
        assert_eq!(result.converted_amount, dec!(0.00));
    }

    #[test]
    fn test_ves_to_usd_rate_of_one() {
        let result = convert_to_usd(dec!(100.00), dec!(1.00), test_date()).unwrap();
        assert_eq!(result.converted_amount, dec!(100.00));
    }

    #[test]
    fn test_ves_to_usd_very_high_rate() {
        // Large VES amount divided by high rate
        let result = convert_to_usd(dec!(450000000.00), dec!(4500000.00), test_date()).unwrap();
        assert_eq!(result.converted_amount, dec!(100.00));
    }

    #[test]
    fn test_ves_to_usd_small_amount() {
        // 0.36 / 36.50 = 0.00986... -> rounds to 0.01
        let result = convert_to_usd(dec!(0.36), dec!(36.50), test_date()).unwrap();
        assert_eq!(result.converted_amount, dec!(0.01));
    }

    #[test]
    fn test_ves_to_usd_negative_amount_error() {
        let result = convert_to_usd(dec!(-100.00), dec!(36.50), test_date());
        assert!(result.is_err());
    }

    #[test]
    fn test_ves_to_usd_zero_rate_error() {
        let result = convert_to_usd(dec!(100.00), dec!(0.00), test_date());
        assert!(result.is_err());
    }

    // --- Round-trip tests ---

    #[test]
    fn test_round_trip_usd_ves_usd() {
        let rate = dec!(36.50);
        let original = dec!(1000.00);
        let to_ves = convert_to_ves(original, rate, test_date()).unwrap();
        let back_to_usd = convert_to_usd(to_ves.converted_amount, rate, test_date()).unwrap();
        assert_eq!(back_to_usd.converted_amount, original);
    }

    // --- Currency Display ---

    #[test]
    fn test_currency_display() {
        assert_eq!(format!("{}", Currency::VES), "VES");
        assert_eq!(format!("{}", Currency::USD), "USD");
    }

    // --- Date preservation ---

    #[test]
    fn test_rate_date_preserved() {
        let date = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let result = convert_to_ves(dec!(100.00), dec!(36.50), date).unwrap();
        assert_eq!(result.rate_date, date);
    }
}
