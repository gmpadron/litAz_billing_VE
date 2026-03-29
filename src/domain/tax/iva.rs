//! Cálculos de IVA (Impuesto al Valor Agregado) conforme al SENIAT.
//!
//! Alícuotas vigentes:
//! - General: 16%
//! - Reducida: 8%
//! - Lujo (adicional): 31%
//! - Exento: 0%

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

/// Errores posibles en los cálculos de IVA.
#[derive(Debug, Error)]
pub enum IvaError {
    #[error("El monto base no puede ser negativo: {0}")]
    NegativeBaseAmount(Decimal),
}

/// Alícuota de IVA aplicable según la normativa SENIAT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IvaRate {
    /// Alícuota general: 16%
    General,
    /// Alícuota reducida: 8%
    Reduced,
    /// Alícuota adicional para bienes de lujo: 31%
    Luxury,
    /// Exento de IVA: 0%
    Exempt,
}

impl IvaRate {
    /// Retorna la alícuota como un `Decimal` (ej: 0.16 para 16%).
    pub fn rate(&self) -> Decimal {
        match self {
            IvaRate::General => dec!(0.16),
            IvaRate::Reduced => dec!(0.08),
            IvaRate::Luxury => dec!(0.31),
            IvaRate::Exempt => dec!(0.00),
        }
    }

    /// Retorna la alícuota como porcentaje entero (ej: 16 para 16%).
    pub fn percentage(&self) -> Decimal {
        match self {
            IvaRate::General => dec!(16),
            IvaRate::Reduced => dec!(8),
            IvaRate::Luxury => dec!(31),
            IvaRate::Exempt => dec!(0),
        }
    }
}

/// Resultado del cálculo de IVA para un monto individual.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IvaResult {
    /// Monto base (sin impuesto).
    pub base_amount: Decimal,
    /// Alícuota aplicada (ej: 0.16).
    pub tax_rate: Decimal,
    /// Monto del impuesto calculado.
    pub tax_amount: Decimal,
    /// Total (base + impuesto).
    pub total: Decimal,
}

/// Desglose de IVA para múltiples items con distintas alícuotas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IvaBreakdown {
    /// Detalle por cada item.
    pub items: Vec<IvaResult>,
    /// Suma de todos los montos base.
    pub total_base: Decimal,
    /// Suma de todos los impuestos.
    pub total_tax: Decimal,
    /// Gran total (base + impuestos).
    pub grand_total: Decimal,
}

/// Calcula el IVA para un monto base y una alícuota dada.
///
/// El redondeo a 2 decimales se aplica al final del cálculo,
/// nunca en pasos intermedios.
///
/// # Argumentos
///
/// * `base` - Monto base imponible (debe ser >= 0).
/// * `rate` - Alícuota de IVA a aplicar.
///
/// # Errores
///
/// Retorna `IvaError::NegativeBaseAmount` si el monto base es negativo.
///
/// # Ejemplo
///
/// ```
/// use rust_decimal::Decimal;
/// use rust_decimal_macros::dec;
/// use billing_core::domain::tax::iva::{calculate_iva, IvaRate};
///
/// let result = calculate_iva(dec!(100.00), IvaRate::General).unwrap();
/// assert_eq!(result.tax_amount, dec!(16.00));
/// assert_eq!(result.total, dec!(116.00));
/// ```
pub fn calculate_iva(base: Decimal, rate: IvaRate) -> Result<IvaResult, IvaError> {
    if base < Decimal::ZERO {
        return Err(IvaError::NegativeBaseAmount(base));
    }

    let tax_rate = rate.rate();
    let tax_amount = (base * tax_rate).round_dp(2);
    let total = (base + tax_amount).round_dp(2);

    Ok(IvaResult {
        base_amount: base,
        tax_rate,
        tax_amount,
        total,
    })
}

/// Calcula el desglose de IVA para múltiples items, cada uno con su propia alícuota.
///
/// Útil para facturas con items que tienen distintas alícuotas (ej: algunos exentos,
/// otros con alícuota general).
///
/// # Argumentos
///
/// * `items` - Slice de tuplas `(monto_base, alícuota)`.
///
/// # Errores
///
/// Retorna error si algún monto base es negativo.
pub fn calculate_iva_breakdown(items: &[(Decimal, IvaRate)]) -> Result<IvaBreakdown, IvaError> {
    let mut results = Vec::with_capacity(items.len());
    let mut total_base = Decimal::ZERO;
    let mut total_tax = Decimal::ZERO;

    for &(base, rate) in items {
        let result = calculate_iva(base, rate)?;
        total_base += result.base_amount;
        total_tax += result.tax_amount;
        results.push(result);
    }

    let grand_total = (total_base + total_tax).round_dp(2);

    Ok(IvaBreakdown {
        items: results,
        total_base,
        total_tax,
        grand_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // --- IvaRate tests ---

    #[test]
    fn test_iva_rate_general() {
        assert_eq!(IvaRate::General.rate(), dec!(0.16));
        assert_eq!(IvaRate::General.percentage(), dec!(16));
    }

    #[test]
    fn test_iva_rate_reduced() {
        assert_eq!(IvaRate::Reduced.rate(), dec!(0.08));
        assert_eq!(IvaRate::Reduced.percentage(), dec!(8));
    }

    #[test]
    fn test_iva_rate_luxury() {
        assert_eq!(IvaRate::Luxury.rate(), dec!(0.31));
        assert_eq!(IvaRate::Luxury.percentage(), dec!(31));
    }

    #[test]
    fn test_iva_rate_exempt() {
        assert_eq!(IvaRate::Exempt.rate(), dec!(0.00));
        assert_eq!(IvaRate::Exempt.percentage(), dec!(0));
    }

    // --- calculate_iva tests ---

    #[test]
    fn test_iva_general_16_percent() {
        let result = calculate_iva(dec!(100.00), IvaRate::General).unwrap();
        assert_eq!(result.base_amount, dec!(100.00));
        assert_eq!(result.tax_rate, dec!(0.16));
        assert_eq!(result.tax_amount, dec!(16.00));
        assert_eq!(result.total, dec!(116.00));
    }

    #[test]
    fn test_iva_reduced_8_percent() {
        let result = calculate_iva(dec!(250.50), IvaRate::Reduced).unwrap();
        assert_eq!(result.tax_amount, dec!(20.04));
        assert_eq!(result.total, dec!(270.54));
    }

    #[test]
    fn test_iva_luxury_31_percent() {
        let result = calculate_iva(dec!(1000.00), IvaRate::Luxury).unwrap();
        assert_eq!(result.tax_amount, dec!(310.00));
        assert_eq!(result.total, dec!(1310.00));
    }

    #[test]
    fn test_iva_exempt_0_percent() {
        let result = calculate_iva(dec!(500.00), IvaRate::Exempt).unwrap();
        assert_eq!(result.tax_amount, dec!(0.00));
        assert_eq!(result.total, dec!(500.00));
    }

    #[test]
    fn test_iva_zero_base() {
        let result = calculate_iva(dec!(0.00), IvaRate::General).unwrap();
        assert_eq!(result.tax_amount, dec!(0.00));
        assert_eq!(result.total, dec!(0.00));
    }

    #[test]
    fn test_iva_negative_base_error() {
        let result = calculate_iva(dec!(-100.00), IvaRate::General);
        assert!(result.is_err());
    }

    #[test]
    fn test_iva_very_small_amount() {
        let result = calculate_iva(dec!(0.01), IvaRate::General).unwrap();
        // 0.01 * 0.16 = 0.0016, rounded to 0.00
        assert_eq!(result.tax_amount, dec!(0.00));
        assert_eq!(result.total, dec!(0.01));
    }

    #[test]
    fn test_iva_very_large_amount() {
        let result = calculate_iva(dec!(999999999.99), IvaRate::General).unwrap();
        assert_eq!(result.tax_amount, dec!(160000000.00));
        assert_eq!(result.total, dec!(1159999999.99));
    }

    #[test]
    fn test_iva_rounding() {
        // 33.33 * 0.16 = 5.3328 -> rounds to 5.33
        let result = calculate_iva(dec!(33.33), IvaRate::General).unwrap();
        assert_eq!(result.tax_amount, dec!(5.33));
        assert_eq!(result.total, dec!(38.66));
    }

    #[test]
    fn test_iva_rounding_half_up() {
        // 156.25 * 0.08 = 12.50 (exact)
        let result = calculate_iva(dec!(156.25), IvaRate::Reduced).unwrap();
        assert_eq!(result.tax_amount, dec!(12.50));
        assert_eq!(result.total, dec!(168.75));
    }

    // --- calculate_iva_breakdown tests ---

    #[test]
    fn test_breakdown_single_item() {
        let items = vec![(dec!(100.00), IvaRate::General)];
        let breakdown = calculate_iva_breakdown(&items).unwrap();
        assert_eq!(breakdown.items.len(), 1);
        assert_eq!(breakdown.total_base, dec!(100.00));
        assert_eq!(breakdown.total_tax, dec!(16.00));
        assert_eq!(breakdown.grand_total, dec!(116.00));
    }

    #[test]
    fn test_breakdown_mixed_rates() {
        let items = vec![
            (dec!(100.00), IvaRate::General), // tax: 16.00
            (dec!(200.00), IvaRate::Reduced), // tax: 16.00
            (dec!(50.00), IvaRate::Exempt),   // tax: 0.00
            (dec!(300.00), IvaRate::Luxury),  // tax: 93.00
        ];
        let breakdown = calculate_iva_breakdown(&items).unwrap();
        assert_eq!(breakdown.items.len(), 4);
        assert_eq!(breakdown.total_base, dec!(650.00));
        assert_eq!(breakdown.total_tax, dec!(125.00));
        assert_eq!(breakdown.grand_total, dec!(775.00));
    }

    #[test]
    fn test_breakdown_empty_items() {
        let items: Vec<(Decimal, IvaRate)> = vec![];
        let breakdown = calculate_iva_breakdown(&items).unwrap();
        assert_eq!(breakdown.items.len(), 0);
        assert_eq!(breakdown.total_base, dec!(0));
        assert_eq!(breakdown.total_tax, dec!(0));
        assert_eq!(breakdown.grand_total, dec!(0));
    }

    #[test]
    fn test_breakdown_all_exempt() {
        let items = vec![
            (dec!(100.00), IvaRate::Exempt),
            (dec!(200.00), IvaRate::Exempt),
        ];
        let breakdown = calculate_iva_breakdown(&items).unwrap();
        assert_eq!(breakdown.total_tax, dec!(0.00));
        assert_eq!(breakdown.grand_total, dec!(300.00));
    }

    #[test]
    fn test_breakdown_negative_base_propagates_error() {
        let items = vec![
            (dec!(100.00), IvaRate::General),
            (dec!(-50.00), IvaRate::General),
        ];
        let result = calculate_iva_breakdown(&items);
        assert!(result.is_err());
    }
}
