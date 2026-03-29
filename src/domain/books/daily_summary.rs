//! Resumen diario de ventas a consumidores finales sin RIF.
//!
//! Segun la normativa SENIAT, las ventas a consumidores finales sin RIF
//! pueden agruparse en un resumen diario en el Libro de Ventas, en lugar
//! de registrar cada factura individualmente.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::SalesBookEntry;

/// Resumen diario de ventas a consumidores finales.
///
/// Agrupa todas las ventas de un dia a consumidores finales sin RIF
/// en una sola entrada para el Libro de Ventas.
#[derive(Debug, Clone)]
pub struct DailySummary {
    /// Fecha del resumen.
    pub date: NaiveDate,
    /// Cantidad total de facturas agrupadas en este resumen.
    pub total_invoices: u32,
    /// Monto total de todas las facturas del dia (incluyendo IVA).
    pub total_amount: Decimal,
    /// Suma de bases imponibles exentas.
    pub exempt_base: Decimal,
    /// Suma de bases imponibles gravadas con alicuota general (16%).
    pub general_base: Decimal,
    /// Suma del IVA general.
    pub general_tax: Decimal,
    /// Suma de bases imponibles gravadas con alicuota reducida (8%).
    pub reduced_base: Decimal,
    /// Suma del IVA reducido.
    pub reduced_tax: Decimal,
    /// Suma de bases imponibles gravadas con alicuota de lujo (31%).
    pub luxury_base: Decimal,
    /// Suma del IVA de lujo.
    pub luxury_tax: Decimal,
}

impl DailySummary {
    /// Crea un resumen diario a partir de un conjunto de entradas del libro de ventas.
    ///
    /// Solo incluye las entradas que corresponden a consumidores finales
    /// (sin RIF) para la fecha indicada. Las entradas que no son de
    /// consumidores finales o que tienen una fecha diferente son ignoradas.
    ///
    /// # Argumentos
    ///
    /// * `date` - Fecha para la cual generar el resumen.
    /// * `entries` - Conjunto de entradas del libro de ventas a agregar.
    ///
    /// # Retorna
    ///
    /// Un `DailySummary` con los totales agregados del dia.
    pub fn from_entries(date: NaiveDate, entries: &[SalesBookEntry]) -> DailySummary {
        let relevant: Vec<&SalesBookEntry> = entries
            .iter()
            .filter(|e| e.entry_date == date && e.is_consumer_final())
            .collect();

        let total_invoices = relevant.len() as u32;
        let total_amount = relevant.iter().map(|e| e.total_amount).sum();
        let exempt_base = relevant.iter().map(|e| e.exempt_base).sum();
        let general_base = relevant.iter().map(|e| e.general_base).sum();
        let general_tax = relevant.iter().map(|e| e.general_tax).sum();
        let reduced_base = relevant.iter().map(|e| e.reduced_base).sum();
        let reduced_tax = relevant.iter().map(|e| e.reduced_tax).sum();
        let luxury_base = relevant.iter().map(|e| e.luxury_base).sum();
        let luxury_tax = relevant.iter().map(|e| e.luxury_tax).sum();

        DailySummary {
            date,
            total_invoices,
            total_amount,
            exempt_base,
            general_base,
            general_tax,
            reduced_base,
            reduced_tax,
            luxury_base,
            luxury_tax,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn make_consumer_entry(
        date: NaiveDate,
        total: Decimal,
        general_base: Decimal,
        general_tax: Decimal,
    ) -> SalesBookEntry {
        SalesBookEntry {
            entry_date: date,
            buyer_name: "Consumidor Final".to_string(),
            buyer_rif: None,
            invoice_number: "00000001".to_string(),
            control_number: "00-00000001".to_string(),
            total_amount: total,
            exempt_base: dec!(0.00),
            general_base,
            general_tax,
            reduced_base: dec!(0.00),
            reduced_tax: dec!(0.00),
            luxury_base: dec!(0.00),
            luxury_tax: dec!(0.00),
            is_summary: false,
            period: "2026-03".to_string(),
        }
    }

    fn make_rif_entry(date: NaiveDate) -> SalesBookEntry {
        SalesBookEntry {
            entry_date: date,
            buyer_name: "Empresa ABC".to_string(),
            buyer_rif: Some("J-12345678-9".to_string()),
            invoice_number: "00000010".to_string(),
            control_number: "00-00000010".to_string(),
            total_amount: dec!(50000.00),
            exempt_base: dec!(0.00),
            general_base: dec!(43103.45),
            general_tax: dec!(6896.55),
            reduced_base: dec!(0.00),
            reduced_tax: dec!(0.00),
            luxury_base: dec!(0.00),
            luxury_tax: dec!(0.00),
            is_summary: false,
            period: "2026-03".to_string(),
        }
    }

    #[test]
    fn test_empty_entries() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let summary = DailySummary::from_entries(date, &[]);
        assert_eq!(summary.total_invoices, 0);
        assert_eq!(summary.total_amount, dec!(0));
        assert_eq!(summary.general_base, dec!(0));
        assert_eq!(summary.general_tax, dec!(0));
    }

    #[test]
    fn test_single_entry() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let entry = make_consumer_entry(date, dec!(1160.00), dec!(1000.00), dec!(160.00));
        let summary = DailySummary::from_entries(date, &[entry]);

        assert_eq!(summary.total_invoices, 1);
        assert_eq!(summary.total_amount, dec!(1160.00));
        assert_eq!(summary.general_base, dec!(1000.00));
        assert_eq!(summary.general_tax, dec!(160.00));
    }

    #[test]
    fn test_multiple_entries_aggregation() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let entries = vec![
            make_consumer_entry(date, dec!(1160.00), dec!(1000.00), dec!(160.00)),
            make_consumer_entry(date, dec!(2320.00), dec!(2000.00), dec!(320.00)),
            make_consumer_entry(date, dec!(580.00), dec!(500.00), dec!(80.00)),
        ];

        let summary = DailySummary::from_entries(date, &entries);

        assert_eq!(summary.total_invoices, 3);
        assert_eq!(summary.total_amount, dec!(4060.00));
        assert_eq!(summary.general_base, dec!(3500.00));
        assert_eq!(summary.general_tax, dec!(560.00));
    }

    #[test]
    fn test_excludes_rif_entries() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let entries = vec![
            make_consumer_entry(date, dec!(1160.00), dec!(1000.00), dec!(160.00)),
            make_rif_entry(date),
            make_consumer_entry(date, dec!(580.00), dec!(500.00), dec!(80.00)),
        ];

        let summary = DailySummary::from_entries(date, &entries);

        assert_eq!(summary.total_invoices, 2);
        assert_eq!(summary.total_amount, dec!(1740.00));
    }

    #[test]
    fn test_excludes_different_date() {
        let target_date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let other_date = NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();

        let entries = vec![
            make_consumer_entry(target_date, dec!(1160.00), dec!(1000.00), dec!(160.00)),
            make_consumer_entry(other_date, dec!(9999.00), dec!(8620.69), dec!(1378.31)),
        ];

        let summary = DailySummary::from_entries(target_date, &entries);

        assert_eq!(summary.total_invoices, 1);
        assert_eq!(summary.total_amount, dec!(1160.00));
    }

    #[test]
    fn test_mixed_entries_only_consumer_final_counted() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let other_date = NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();

        let entries = vec![
            make_consumer_entry(date, dec!(1160.00), dec!(1000.00), dec!(160.00)),
            make_rif_entry(date),
            make_consumer_entry(other_date, dec!(580.00), dec!(500.00), dec!(80.00)),
            make_consumer_entry(date, dec!(2320.00), dec!(2000.00), dec!(320.00)),
        ];

        let summary = DailySummary::from_entries(date, &entries);

        assert_eq!(summary.total_invoices, 2);
        assert_eq!(summary.total_amount, dec!(3480.00));
        assert_eq!(summary.general_base, dec!(3000.00));
        assert_eq!(summary.general_tax, dec!(480.00));
    }

    #[test]
    fn test_date_preserved() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let summary = DailySummary::from_entries(date, &[]);
        assert_eq!(summary.date, date);
    }

    #[test]
    fn test_exempt_entries_aggregation() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let mut entry1 = make_consumer_entry(date, dec!(500.00), dec!(0.00), dec!(0.00));
        entry1.exempt_base = dec!(500.00);

        let mut entry2 = make_consumer_entry(date, dec!(300.00), dec!(0.00), dec!(0.00));
        entry2.exempt_base = dec!(300.00);

        let summary = DailySummary::from_entries(date, &[entry1, entry2]);

        assert_eq!(summary.total_invoices, 2);
        assert_eq!(summary.exempt_base, dec!(800.00));
        assert_eq!(summary.general_base, dec!(0.00));
    }

    #[test]
    fn test_reduced_rate_aggregation() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let mut entry = make_consumer_entry(date, dec!(1080.00), dec!(0.00), dec!(0.00));
        entry.reduced_base = dec!(1000.00);
        entry.reduced_tax = dec!(80.00);

        let summary = DailySummary::from_entries(date, &[entry]);

        assert_eq!(summary.reduced_base, dec!(1000.00));
        assert_eq!(summary.reduced_tax, dec!(80.00));
    }
}
