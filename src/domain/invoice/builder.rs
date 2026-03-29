//! Construcción y cálculo de facturas fiscales con patrón Builder.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::domain::tax::iva::{IvaRate, calculate_iva};

use super::validation::{ValidationError, validate_invoice_data};

/// Condición de pago del documento fiscal.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PaymentCondition {
    /// Pago al contado.
    #[default]
    Cash,
    /// Pago a crédito con plazo en días.
    Credit { days: Option<u32> },
}

/// Ítem individual de una factura (entidad de dominio puro).
#[derive(Debug, Clone)]
pub struct InvoiceItemData {
    /// Descripción del bien o servicio.
    pub description: String,
    /// Cantidad (debe ser > 0).
    pub quantity: Decimal,
    /// Precio unitario (debe ser >= 0).
    pub unit_price: Decimal,
    /// Alícuota de IVA aplicable.
    pub tax_rate: IvaRate,
}

impl InvoiceItemData {
    /// Calcula el subtotal del ítem (cantidad * precio unitario), sin redondeo intermedio.
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

/// Totales calculados de una factura.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceTotals {
    /// Suma de todos los subtotales (sin IVA).
    pub subtotal: Decimal,
    /// Monto de IVA a alícuota general (16%).
    pub tax_general: Decimal,
    /// Monto de IVA a alícuota reducida (8%).
    pub tax_reduced: Decimal,
    /// Monto de IVA a alícuota de lujo (31%).
    pub tax_luxury: Decimal,
    /// Total de IVA (suma de todas las alícuotas).
    pub total_tax: Decimal,
    /// Gran total (subtotal + IVA total).
    pub grand_total: Decimal,
}

/// Datos de una factura fiscal (entidad de dominio puro, sin dependencias de framework).
#[derive(Debug, Clone)]
pub struct InvoiceData {
    /// Número de factura secuencial.
    pub invoice_number: String,
    /// Número de control (formato `00-XXXXXXXX`).
    pub control_number: String,
    /// Fecha de emisión.
    pub invoice_date: NaiveDate,
    /// RIF del cliente (None si es consumidor final sin RIF).
    pub client_rif: Option<String>,
    /// Nombre o razón social del cliente.
    pub client_name: String,
    /// Domicilio fiscal del cliente.
    pub client_address: String,
    /// Ítems de la factura.
    pub items: Vec<InvoiceItemData>,
    /// Condición de pago.
    pub payment_condition: PaymentCondition,
    /// Indica si aplica la leyenda "SIN DERECHO A CRÉDITO FISCAL" (consumidor final).
    pub no_fiscal_credit: bool,
    /// Moneda de la operación (ej: "USD", "VES").
    pub currency: String,
    /// Tasa de cambio BCV del día de la operación.
    pub exchange_rate: Decimal,
}

impl InvoiceData {
    /// Calcula los totales de la factura.
    pub fn calculate_totals(&self) -> InvoiceTotals {
        let mut subtotal = Decimal::ZERO;
        let mut tax_general = Decimal::ZERO;
        let mut tax_reduced = Decimal::ZERO;
        let mut tax_luxury = Decimal::ZERO;

        for item in &self.items {
            let item_subtotal = item.subtotal();
            subtotal += item_subtotal;

            // calculate_iva is infallible for non-negative bases; items are validated before build
            if let Ok(iva) = calculate_iva(item_subtotal, item.tax_rate) {
                match item.tax_rate {
                    IvaRate::General => tax_general += iva.tax_amount,
                    IvaRate::Reduced => tax_reduced += iva.tax_amount,
                    IvaRate::Luxury => tax_luxury += iva.tax_amount,
                    IvaRate::Exempt => {}
                }
            }
        }

        let total_tax = (tax_general + tax_reduced + tax_luxury).round_dp(2);
        let grand_total = (subtotal + total_tax).round_dp(2);
        let subtotal = subtotal.round_dp(2);

        InvoiceTotals {
            subtotal,
            tax_general: tax_general.round_dp(2),
            tax_reduced: tax_reduced.round_dp(2),
            tax_luxury: tax_luxury.round_dp(2),
            total_tax,
            grand_total,
        }
    }
}

/// Constructor de facturas fiscales con patrón Builder.
#[derive(Debug, Default)]
pub struct InvoiceBuilder {
    invoice_number: String,
    control_number: String,
    invoice_date: Option<NaiveDate>,
    client_rif: Option<String>,
    client_name: String,
    client_address: String,
    items: Vec<InvoiceItemData>,
    payment_condition: PaymentCondition,
    no_fiscal_credit: bool,
    currency: String,
    exchange_rate: Decimal,
}

impl InvoiceBuilder {
    /// Crea un nuevo builder vacío.
    pub fn new() -> Self {
        Self {
            currency: "USD".to_string(),
            exchange_rate: dec!(1),
            ..Default::default()
        }
    }

    /// Establece el número de factura.
    pub fn invoice_number(mut self, number: impl Into<String>) -> Self {
        self.invoice_number = number.into();
        self
    }

    /// Establece el número de control.
    pub fn control_number(mut self, number: impl Into<String>) -> Self {
        self.control_number = number.into();
        self
    }

    /// Establece la fecha de emisión.
    pub fn invoice_date(mut self, date: NaiveDate) -> Self {
        self.invoice_date = Some(date);
        self
    }

    /// Establece los datos del cliente.
    pub fn client(
        mut self,
        rif: Option<impl Into<String>>,
        name: impl Into<String>,
        address: impl Into<String>,
    ) -> Self {
        self.client_rif = rif.map(|r| r.into());
        self.client_name = name.into();
        self.client_address = address.into();
        // Si no tiene RIF es consumidor final, forzar bandera
        if self.client_rif.is_none() {
            self.no_fiscal_credit = true;
        }
        self
    }

    /// Agrega un ítem a la factura.
    pub fn add_item(
        mut self,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
        tax_rate: IvaRate,
    ) -> Self {
        self.items.push(InvoiceItemData {
            description: description.into(),
            quantity,
            unit_price,
            tax_rate,
        });
        self
    }

    /// Establece la condición de pago.
    pub fn payment_condition(mut self, condition: PaymentCondition) -> Self {
        self.payment_condition = condition;
        self
    }

    /// Establece la moneda y tasa de cambio BCV.
    pub fn currency(mut self, currency: impl Into<String>, exchange_rate: Decimal) -> Self {
        self.currency = currency.into();
        self.exchange_rate = exchange_rate;
        self
    }

    /// Marca explícitamente como "SIN DERECHO A CRÉDITO FISCAL".
    pub fn no_fiscal_credit(mut self) -> Self {
        self.no_fiscal_credit = true;
        self
    }

    /// Construye la factura, ejecutando todas las validaciones SENIAT.
    ///
    /// Retorna `Ok(InvoiceData)` si todos los datos son válidos,
    /// o `Err(Vec<ValidationError>)` con todos los errores encontrados.
    pub fn build(self) -> Result<InvoiceData, Vec<ValidationError>> {
        let date = self
            .invoice_date
            .unwrap_or_else(|| chrono::Local::now().date_naive());

        let data = InvoiceData {
            invoice_number: self.invoice_number,
            control_number: self.control_number,
            invoice_date: date,
            client_rif: self.client_rif,
            client_name: self.client_name,
            client_address: self.client_address,
            items: self.items,
            payment_condition: self.payment_condition,
            no_fiscal_credit: self.no_fiscal_credit,
            currency: self.currency,
            exchange_rate: self.exchange_rate,
        };

        let errors = validate_invoice_data(&data);
        if errors.is_empty() {
            Ok(data)
        } else {
            Err(errors)
        }
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
    fn test_build_valid_invoice() {
        let invoice = InvoiceBuilder::new()
            .invoice_number("00000001")
            .control_number("00-00000001")
            .invoice_date(sample_date())
            .client(
                Some("J-12345678-9"),
                "Empresa Prueba C.A.",
                "Av. Principal, Caracas",
            )
            .add_item(
                "Servicio consultoría",
                dec!(2),
                dec!(500.00),
                IvaRate::General,
            )
            .payment_condition(PaymentCondition::Cash)
            .currency("USD", dec!(36.50))
            .build();

        assert!(invoice.is_ok(), "Expected Ok, got: {:?}", invoice.err());
    }

    #[test]
    fn test_build_invalid_invoice_returns_errors() {
        let result = InvoiceBuilder::new()
            .invoice_number("")
            .control_number("")
            .invoice_date(sample_date())
            .client(Some("J-12345678-9"), "", "")
            .currency("USD", dec!(36.50))
            .build();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_calculate_totals_single_item_general_iva() {
        let invoice = InvoiceBuilder::new()
            .invoice_number("00000001")
            .control_number("00-00000001")
            .invoice_date(sample_date())
            .client(Some("J-12345678-9"), "Empresa Prueba C.A.", "Av. Principal")
            .add_item("Producto", dec!(1), dec!(100.00), IvaRate::General)
            .currency("USD", dec!(1))
            .build()
            .unwrap();

        let totals = invoice.calculate_totals();
        assert_eq!(totals.subtotal, dec!(100.00));
        assert_eq!(totals.tax_general, dec!(16.00));
        assert_eq!(totals.total_tax, dec!(16.00));
        assert_eq!(totals.grand_total, dec!(116.00));
    }

    #[test]
    fn test_calculate_totals_mixed_rates() {
        let invoice = InvoiceBuilder::new()
            .invoice_number("00000001")
            .control_number("00-00000001")
            .invoice_date(sample_date())
            .client(Some("J-12345678-9"), "Empresa Prueba C.A.", "Av. Principal")
            .add_item("Item General", dec!(1), dec!(100.00), IvaRate::General)
            .add_item("Item Reducido", dec!(1), dec!(200.00), IvaRate::Reduced)
            .add_item("Item Exento", dec!(1), dec!(50.00), IvaRate::Exempt)
            .currency("USD", dec!(1))
            .build()
            .unwrap();

        let totals = invoice.calculate_totals();
        assert_eq!(totals.subtotal, dec!(350.00));
        assert_eq!(totals.tax_general, dec!(16.00));
        assert_eq!(totals.tax_reduced, dec!(16.00));
        assert_eq!(totals.total_tax, dec!(32.00));
        assert_eq!(totals.grand_total, dec!(382.00));
    }

    #[test]
    fn test_consumer_final_auto_no_fiscal_credit() {
        let invoice = InvoiceBuilder::new()
            .invoice_number("00000001")
            .control_number("00-00000001")
            .invoice_date(sample_date())
            .client(
                None::<String>,
                "Juan Consumidor",
                "Calle Principal, Caracas",
            )
            .add_item("Producto", dec!(1), dec!(50.00), IvaRate::General)
            .currency("USD", dec!(1))
            .build()
            .unwrap();

        assert!(invoice.no_fiscal_credit);
        assert!(invoice.client_rif.is_none());
    }

    #[test]
    fn test_credit_payment_valid() {
        let invoice = InvoiceBuilder::new()
            .invoice_number("00000001")
            .control_number("00-00000001")
            .invoice_date(sample_date())
            .client(Some("J-12345678-9"), "Empresa Prueba C.A.", "Av. Principal")
            .add_item("Producto", dec!(1), dec!(100.00), IvaRate::General)
            .payment_condition(PaymentCondition::Credit { days: Some(30) })
            .currency("USD", dec!(1))
            .build();

        assert!(invoice.is_ok());
    }

    #[test]
    fn test_item_subtotal_calculation() {
        let item = InvoiceItemData {
            description: "Test".to_string(),
            quantity: dec!(3),
            unit_price: dec!(10.00),
            tax_rate: IvaRate::General,
        };
        assert_eq!(item.subtotal(), dec!(30.00));
        assert_eq!(item.tax_amount(), dec!(4.80));
        assert_eq!(item.total(), dec!(34.80));
    }
}
