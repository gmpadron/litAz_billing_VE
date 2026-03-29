//! Cálculos de ISLR (Impuesto Sobre la Renta) conforme al Decreto 1.808.
//!
//! Los porcentajes de retención varían según el tipo de actividad económica
//! y el tipo de persona (natural o jurídica).

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

/// Errores posibles en los cálculos de ISLR.
#[derive(Debug, Error)]
pub enum IslrError {
    #[error("El monto base no puede ser negativo: {0}")]
    NegativeBaseAmount(Decimal),
}

/// Tipos de actividad económica para retenciones de ISLR según el Decreto 1.808.
///
/// Cada variante incluye la alícuota aplicable como porcentaje del pago o abono en cuenta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IslrActivity {
    /// Honorarios profesionales persona jurídica domiciliada: 5%
    ProfessionalFeesJuridical,
    /// Honorarios profesionales persona natural residente: 3%
    ProfessionalFeesNatural,
    /// Comisiones mercantiles persona jurídica domiciliada: 5%
    CommissionsJuridical,
    /// Comisiones mercantiles persona natural residente: 3%
    CommissionsNatural,
    /// Intereses de capitales: 3%
    InterestOnCapital,
    /// Alquiler de bienes inmuebles: 3%
    RealEstateRental,
    /// Alquiler de bienes muebles: 3%
    PersonalPropertyRental,
    /// Servicios de transporte: 1%
    TransportServices,
    /// Publicidad y propaganda: 5%
    AdvertisingServices,
    /// Servicios tecnológicos: 5%
    TechnologyServices,
    /// Pagos a contratistas y subcontratistas: 2%
    ContractorServices,
    /// Pagos a no domiciliados/no residentes (tarifa general): 34%
    NonResident,
    /// Actividad no clasificada con tasa personalizada.
    Custom(Decimal),
}

impl IslrActivity {
    /// Retorna la alícuota de retención como fracción decimal (ej: 0.05 para 5%).
    pub fn rate(&self) -> Decimal {
        match self {
            IslrActivity::ProfessionalFeesJuridical => dec!(0.05),
            IslrActivity::ProfessionalFeesNatural => dec!(0.03),
            IslrActivity::CommissionsJuridical => dec!(0.05),
            IslrActivity::CommissionsNatural => dec!(0.03),
            IslrActivity::InterestOnCapital => dec!(0.03),
            IslrActivity::RealEstateRental => dec!(0.03),
            IslrActivity::PersonalPropertyRental => dec!(0.03),
            IslrActivity::TransportServices => dec!(0.01),
            IslrActivity::AdvertisingServices => dec!(0.05),
            IslrActivity::TechnologyServices => dec!(0.05),
            IslrActivity::ContractorServices => dec!(0.02),
            IslrActivity::NonResident => dec!(0.34),
            IslrActivity::Custom(rate) => *rate,
        }
    }

    /// Retorna la alícuota como porcentaje entero (ej: 5 para 5%).
    pub fn percentage(&self) -> Decimal {
        self.rate() * dec!(100)
    }
}

/// Resultado del cálculo de ISLR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IslrResult {
    /// Monto base sobre el cual se calcula el impuesto.
    pub base_amount: Decimal,
    /// Alícuota aplicada como fracción decimal.
    pub rate: Decimal,
    /// Monto del impuesto calculado.
    pub tax_amount: Decimal,
}

/// Calcula el ISLR (Impuesto Sobre la Renta) para un monto base y tipo de actividad.
///
/// El redondeo a 2 decimales se aplica al final del cálculo.
///
/// # Argumentos
///
/// * `base` - Monto base imponible (debe ser >= 0).
/// * `activity` - Tipo de actividad que determina la alícuota.
///
/// # Errores
///
/// Retorna `IslrError::NegativeBaseAmount` si el monto base es negativo.
pub fn calculate_islr(base: Decimal, activity: &IslrActivity) -> Result<IslrResult, IslrError> {
    if base < Decimal::ZERO {
        return Err(IslrError::NegativeBaseAmount(base));
    }

    let rate = activity.rate();
    let tax_amount = (base * rate).round_dp(2);

    Ok(IslrResult {
        base_amount: base,
        rate,
        tax_amount,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_professional_fees_juridical_5_percent() {
        let result =
            calculate_islr(dec!(1000.00), &IslrActivity::ProfessionalFeesJuridical).unwrap();
        assert_eq!(result.rate, dec!(0.05));
        assert_eq!(result.tax_amount, dec!(50.00));
    }

    #[test]
    fn test_professional_fees_natural_3_percent() {
        let result = calculate_islr(dec!(1000.00), &IslrActivity::ProfessionalFeesNatural).unwrap();
        assert_eq!(result.rate, dec!(0.03));
        assert_eq!(result.tax_amount, dec!(30.00));
    }

    #[test]
    fn test_commissions_juridical_5_percent() {
        let result = calculate_islr(dec!(500.00), &IslrActivity::CommissionsJuridical).unwrap();
        assert_eq!(result.tax_amount, dec!(25.00));
    }

    #[test]
    fn test_commissions_natural_3_percent() {
        let result = calculate_islr(dec!(500.00), &IslrActivity::CommissionsNatural).unwrap();
        assert_eq!(result.tax_amount, dec!(15.00));
    }

    #[test]
    fn test_interest_on_capital_3_percent() {
        let result = calculate_islr(dec!(10000.00), &IslrActivity::InterestOnCapital).unwrap();
        assert_eq!(result.tax_amount, dec!(300.00));
    }

    #[test]
    fn test_real_estate_rental_3_percent() {
        let result = calculate_islr(dec!(2000.00), &IslrActivity::RealEstateRental).unwrap();
        assert_eq!(result.tax_amount, dec!(60.00));
    }

    #[test]
    fn test_personal_property_rental_3_percent() {
        let result = calculate_islr(dec!(800.00), &IslrActivity::PersonalPropertyRental).unwrap();
        assert_eq!(result.tax_amount, dec!(24.00));
    }

    #[test]
    fn test_transport_services_1_percent() {
        let result = calculate_islr(dec!(5000.00), &IslrActivity::TransportServices).unwrap();
        assert_eq!(result.rate, dec!(0.01));
        assert_eq!(result.tax_amount, dec!(50.00));
    }

    #[test]
    fn test_advertising_services_5_percent() {
        let result = calculate_islr(dec!(3000.00), &IslrActivity::AdvertisingServices).unwrap();
        assert_eq!(result.tax_amount, dec!(150.00));
    }

    #[test]
    fn test_technology_services_5_percent() {
        let result = calculate_islr(dec!(7500.00), &IslrActivity::TechnologyServices).unwrap();
        assert_eq!(result.tax_amount, dec!(375.00));
    }

    #[test]
    fn test_contractor_services_2_percent() {
        let result = calculate_islr(dec!(25000.00), &IslrActivity::ContractorServices).unwrap();
        assert_eq!(result.rate, dec!(0.02));
        assert_eq!(result.tax_amount, dec!(500.00));
    }

    #[test]
    fn test_non_resident_34_percent() {
        let result = calculate_islr(dec!(10000.00), &IslrActivity::NonResident).unwrap();
        assert_eq!(result.rate, dec!(0.34));
        assert_eq!(result.tax_amount, dec!(3400.00));
    }

    #[test]
    fn test_custom_rate() {
        let activity = IslrActivity::Custom(dec!(0.12));
        let result = calculate_islr(dec!(1000.00), &activity).unwrap();
        assert_eq!(result.rate, dec!(0.12));
        assert_eq!(result.tax_amount, dec!(120.00));
    }

    #[test]
    fn test_zero_base() {
        let result = calculate_islr(dec!(0.00), &IslrActivity::NonResident).unwrap();
        assert_eq!(result.tax_amount, dec!(0.00));
    }

    #[test]
    fn test_negative_base_error() {
        let result = calculate_islr(dec!(-100.00), &IslrActivity::ProfessionalFeesJuridical);
        assert!(result.is_err());
    }

    #[test]
    fn test_very_large_amount() {
        let result = calculate_islr(dec!(999999999.99), &IslrActivity::NonResident).unwrap();
        assert_eq!(result.tax_amount, dec!(340000000.00));
    }

    #[test]
    fn test_very_small_amount() {
        let result = calculate_islr(dec!(0.01), &IslrActivity::TransportServices).unwrap();
        // 0.01 * 0.01 = 0.0001 -> rounds to 0.00
        assert_eq!(result.tax_amount, dec!(0.00));
    }

    #[test]
    fn test_rounding() {
        // 333.33 * 0.03 = 9.9999 -> rounds to 10.00
        let result = calculate_islr(dec!(333.33), &IslrActivity::ProfessionalFeesNatural).unwrap();
        assert_eq!(result.tax_amount, dec!(10.00));
    }

    #[test]
    fn test_percentage_display() {
        assert_eq!(
            IslrActivity::ProfessionalFeesJuridical.percentage(),
            dec!(5)
        );
        assert_eq!(IslrActivity::TransportServices.percentage(), dec!(1));
        assert_eq!(IslrActivity::NonResident.percentage(), dec!(34));
    }
}
