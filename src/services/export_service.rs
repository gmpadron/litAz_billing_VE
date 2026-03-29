//! Servicio de exportación de libros de compras y ventas.
//!
//! Genera archivos CSV/TXT con los libros fiscales listos para presentar
//! ante el SENIAT o para importar en software contable.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::errors::AppError;

// ─── Data structs ─────────────────────────────────────────────────────────────

/// Entrada del Libro de Compras para exportación.
#[derive(Debug, Clone)]
pub struct PurchaseBookExportEntry {
    pub entry_date: NaiveDate,
    pub supplier_name: String,
    pub supplier_rif: String,
    pub invoice_number: String,
    pub control_number: String,
    pub total_amount: Decimal,
    pub exempt_base: Decimal,
    pub general_base: Decimal,
    pub general_tax: Decimal,
    pub reduced_base: Decimal,
    pub reduced_tax: Decimal,
    pub iva_withheld: Option<Decimal>,
}

/// Datos completos del Libro de Compras para exportación (un periodo).
#[derive(Debug, Clone)]
pub struct PurchaseBookExportData {
    pub period: String,
    pub entries: Vec<PurchaseBookExportEntry>,
}

/// Entrada del Libro de Ventas para exportación.
#[derive(Debug, Clone)]
pub struct SalesBookExportEntry {
    pub entry_date: NaiveDate,
    pub buyer_name: String,
    pub buyer_rif: Option<String>,
    pub invoice_number: String,
    pub control_number: String,
    pub total_amount: Decimal,
    pub exempt_base: Decimal,
    pub general_base: Decimal,
    pub general_tax: Decimal,
    pub reduced_base: Decimal,
    pub reduced_tax: Decimal,
    pub is_summary: bool,
}

/// Datos completos del Libro de Ventas para exportación (un periodo).
#[derive(Debug, Clone)]
pub struct SalesBookExportData {
    pub period: String,
    pub entries: Vec<SalesBookExportEntry>,
}

// ─── CSV helpers ──────────────────────────────────────────────────────────────

/// Escapa un campo CSV: si contiene coma, comillas o saltos de línea, lo encierra en comillas.
fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Formatea un Decimal para CSV (siempre 2 decimales).
fn csv_decimal(d: Decimal) -> String {
    format!("{:.2}", d)
}

/// Formatea un Option<Decimal> para CSV.
fn csv_opt_decimal(d: Option<Decimal>) -> String {
    match d {
        Some(v) => csv_decimal(v),
        None => String::new(),
    }
}

// ─── Export functions ─────────────────────────────────────────────────────────

/// Exporta el Libro de Compras a formato CSV con totales de periodo al final.
///
/// Columnas: Fecha | Proveedor | RIF Proveedor | N° Factura | N° Control |
///           Total | Base Exenta | Base 16% | IVA 16% | Base 8% | IVA 8% | IVA Retenido
pub fn export_purchase_book(data: &PurchaseBookExportData) -> Result<Vec<u8>, AppError> {
    let mut lines: Vec<String> = Vec::new();

    // Encabezado del archivo
    lines.push(format!("# LIBRO DE COMPRAS - Periodo: {}", data.period));
    lines.push(String::new());

    // Encabezado de columnas CSV
    lines.push(
        [
            "Fecha",
            "Proveedor",
            "RIF Proveedor",
            "N° Factura",
            "N° Control",
            "Total",
            "Base Exenta",
            "Base 16%",
            "IVA 16%",
            "Base 8%",
            "IVA 8%",
            "IVA Retenido",
        ]
        .join(","),
    );

    // Filas de datos
    for entry in &data.entries {
        let row = vec![
            csv_field(&entry.entry_date.to_string()),
            csv_field(&entry.supplier_name),
            csv_field(&entry.supplier_rif),
            csv_field(&entry.invoice_number),
            csv_field(&entry.control_number),
            csv_decimal(entry.total_amount),
            csv_decimal(entry.exempt_base),
            csv_decimal(entry.general_base),
            csv_decimal(entry.general_tax),
            csv_decimal(entry.reduced_base),
            csv_decimal(entry.reduced_tax),
            csv_opt_decimal(entry.iva_withheld),
        ];
        lines.push(row.join(","));
    }

    // Línea de totales
    if !data.entries.is_empty() {
        let total_amount: Decimal = data.entries.iter().map(|e| e.total_amount).sum();
        let total_exempt: Decimal = data.entries.iter().map(|e| e.exempt_base).sum();
        let total_general_base: Decimal = data.entries.iter().map(|e| e.general_base).sum();
        let total_general_tax: Decimal = data.entries.iter().map(|e| e.general_tax).sum();
        let total_reduced_base: Decimal = data.entries.iter().map(|e| e.reduced_base).sum();
        let total_reduced_tax: Decimal = data.entries.iter().map(|e| e.reduced_tax).sum();
        let total_withheld: Decimal = data.entries.iter().filter_map(|e| e.iva_withheld).sum();

        lines.push(String::new());
        lines.push(format!(
            "TOTALES,,,,{},{},{},{},{},{},{}",
            csv_decimal(total_amount),
            csv_decimal(total_exempt),
            csv_decimal(total_general_base),
            csv_decimal(total_general_tax),
            csv_decimal(total_reduced_base),
            csv_decimal(total_reduced_tax),
            csv_decimal(total_withheld),
        ));
    }

    let content = lines.join("\n") + "\n";
    Ok(content.into_bytes())
}

/// Exporta el Libro de Ventas a formato CSV con totales de periodo al final.
///
/// Columnas: Fecha | Comprador | RIF Comprador | N° Factura | N° Control |
///           Total | Base Exenta | Base 16% | IVA 16% | Base 8% | IVA 8% | Es Resumen
pub fn export_sales_book(data: &SalesBookExportData) -> Result<Vec<u8>, AppError> {
    let mut lines: Vec<String> = Vec::new();

    // Encabezado del archivo
    lines.push(format!("# LIBRO DE VENTAS - Periodo: {}", data.period));
    lines.push(String::new());

    // Encabezado de columnas CSV
    lines.push(
        [
            "Fecha",
            "Comprador",
            "RIF Comprador",
            "N° Factura",
            "N° Control",
            "Total",
            "Base Exenta",
            "Base 16%",
            "IVA 16%",
            "Base 8%",
            "IVA 8%",
            "Resumen Diario",
        ]
        .join(","),
    );

    // Filas de datos
    for entry in &data.entries {
        let buyer_rif = entry.buyer_rif.as_deref().unwrap_or("Consumidor Final");
        let is_summary_str = if entry.is_summary { "SI" } else { "NO" };

        let row = vec![
            csv_field(&entry.entry_date.to_string()),
            csv_field(&entry.buyer_name),
            csv_field(buyer_rif),
            csv_field(&entry.invoice_number),
            csv_field(&entry.control_number),
            csv_decimal(entry.total_amount),
            csv_decimal(entry.exempt_base),
            csv_decimal(entry.general_base),
            csv_decimal(entry.general_tax),
            csv_decimal(entry.reduced_base),
            csv_decimal(entry.reduced_tax),
            is_summary_str.to_string(),
        ];
        lines.push(row.join(","));
    }

    // Línea de totales
    if !data.entries.is_empty() {
        let total_amount: Decimal = data.entries.iter().map(|e| e.total_amount).sum();
        let total_exempt: Decimal = data.entries.iter().map(|e| e.exempt_base).sum();
        let total_general_base: Decimal = data.entries.iter().map(|e| e.general_base).sum();
        let total_general_tax: Decimal = data.entries.iter().map(|e| e.general_tax).sum();
        let total_reduced_base: Decimal = data.entries.iter().map(|e| e.reduced_base).sum();
        let total_reduced_tax: Decimal = data.entries.iter().map(|e| e.reduced_tax).sum();

        lines.push(String::new());
        lines.push(format!(
            "TOTALES,,,,,{},{},{},{},{},{},",
            csv_decimal(total_amount),
            csv_decimal(total_exempt),
            csv_decimal(total_general_base),
            csv_decimal(total_general_tax),
            csv_decimal(total_reduced_base),
            csv_decimal(total_reduced_tax),
        ));
    }

    let content = lines.join("\n") + "\n";
    Ok(content.into_bytes())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_purchase_entry() -> PurchaseBookExportEntry {
        PurchaseBookExportEntry {
            entry_date: NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            supplier_name: "Proveedor ABC C.A.".to_string(),
            supplier_rif: "J-12345678-9".to_string(),
            invoice_number: "00000100".to_string(),
            control_number: "00-00000100".to_string(),
            total_amount: dec!(1160.00),
            exempt_base: dec!(0.00),
            general_base: dec!(1000.00),
            general_tax: dec!(160.00),
            reduced_base: dec!(0.00),
            reduced_tax: dec!(0.00),
            iva_withheld: Some(dec!(120.00)),
        }
    }

    fn sample_sales_entry() -> SalesBookExportEntry {
        SalesBookExportEntry {
            entry_date: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            buyer_name: "Cliente XYZ S.A.".to_string(),
            buyer_rif: Some("J-98765432-1".to_string()),
            invoice_number: "00000001".to_string(),
            control_number: "00-00000001".to_string(),
            total_amount: dec!(580.00),
            exempt_base: dec!(0.00),
            general_base: dec!(500.00),
            general_tax: dec!(80.00),
            reduced_base: dec!(0.00),
            reduced_tax: dec!(0.00),
            is_summary: false,
        }
    }

    #[test]
    fn test_export_purchase_book_csv_structure() {
        let data = PurchaseBookExportData {
            period: "2026-03".to_string(),
            entries: vec![sample_purchase_entry()],
        };

        let result = export_purchase_book(&data);
        assert!(result.is_ok());
        let csv = String::from_utf8(result.unwrap()).expect("CSV debe ser UTF-8 válido");

        // Verificar encabezado
        assert!(csv.contains("LIBRO DE COMPRAS"));
        assert!(csv.contains("2026-03"));
        // Verificar columnas
        assert!(csv.contains("Proveedor"));
        assert!(csv.contains("RIF Proveedor"));
        assert!(csv.contains("N° Factura"));
    }

    #[test]
    fn test_export_purchase_book_data_fields() {
        let data = PurchaseBookExportData {
            period: "2026-03".to_string(),
            entries: vec![sample_purchase_entry()],
        };

        let csv = String::from_utf8(export_purchase_book(&data).unwrap()).unwrap();

        assert!(csv.contains("Proveedor ABC C.A."));
        assert!(csv.contains("J-12345678-9"));
        assert!(csv.contains("00000100"));
        assert!(csv.contains("00-00000100"));
        assert!(csv.contains("1160.00"));
        assert!(csv.contains("1000.00"));
        assert!(csv.contains("160.00"));
        assert!(csv.contains("120.00")); // IVA retenido
    }

    #[test]
    fn test_export_purchase_book_totals() {
        let entry1 = sample_purchase_entry();
        let entry2 = PurchaseBookExportEntry {
            entry_date: NaiveDate::from_ymd_opt(2026, 3, 20).unwrap(),
            supplier_name: "Proveedor XYZ".to_string(),
            supplier_rif: "J-99999999-9".to_string(),
            invoice_number: "00000200".to_string(),
            control_number: "00-00000200".to_string(),
            total_amount: dec!(580.00),
            exempt_base: dec!(0.00),
            general_base: dec!(500.00),
            general_tax: dec!(80.00),
            reduced_base: dec!(0.00),
            reduced_tax: dec!(0.00),
            iva_withheld: None,
        };

        let data = PurchaseBookExportData {
            period: "2026-03".to_string(),
            entries: vec![entry1, entry2],
        };

        let csv = String::from_utf8(export_purchase_book(&data).unwrap()).unwrap();
        assert!(csv.contains("TOTALES"));
        assert!(csv.contains("1740.00")); // Total: 1160 + 580
    }

    #[test]
    fn test_export_sales_book_csv_structure() {
        let data = SalesBookExportData {
            period: "2026-03".to_string(),
            entries: vec![sample_sales_entry()],
        };

        let result = export_sales_book(&data);
        assert!(result.is_ok());
        let csv = String::from_utf8(result.unwrap()).unwrap();

        assert!(csv.contains("LIBRO DE VENTAS"));
        assert!(csv.contains("2026-03"));
        assert!(csv.contains("Comprador"));
        assert!(csv.contains("RIF Comprador"));
        assert!(csv.contains("Resumen Diario"));
    }

    #[test]
    fn test_export_sales_book_consumer_final() {
        let entry = SalesBookExportEntry {
            entry_date: NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            buyer_name: "Resumen Diario".to_string(),
            buyer_rif: None, // Consumidor final sin RIF
            invoice_number: "RESUMEN".to_string(),
            control_number: "N/A".to_string(),
            total_amount: dec!(300.00),
            exempt_base: dec!(0.00),
            general_base: dec!(258.62),
            general_tax: dec!(41.38),
            reduced_base: dec!(0.00),
            reduced_tax: dec!(0.00),
            is_summary: true,
        };

        let data = SalesBookExportData {
            period: "2026-03".to_string(),
            entries: vec![entry],
        };

        let csv = String::from_utf8(export_sales_book(&data).unwrap()).unwrap();

        // Consumidor final debe aparecer como "Consumidor Final"
        assert!(csv.contains("Consumidor Final"));
        // El resumen diario debe aparecer como "SI"
        assert!(csv.contains("SI"));
    }

    #[test]
    fn test_export_purchase_book_empty() {
        let data = PurchaseBookExportData {
            period: "2026-03".to_string(),
            entries: vec![],
        };

        let result = export_purchase_book(&data);
        assert!(result.is_ok());
        let csv = String::from_utf8(result.unwrap()).unwrap();

        // Sin entradas, no debe haber línea de TOTALES
        assert!(!csv.contains("TOTALES"));
        assert!(csv.contains("LIBRO DE COMPRAS"));
    }

    #[test]
    fn test_csv_field_escaping_comma() {
        let field_with_comma = csv_field("Empresa, Sociedad y Cia.");
        assert!(field_with_comma.starts_with('"'));
        assert!(field_with_comma.ends_with('"'));
    }

    #[test]
    fn test_csv_field_escaping_quotes() {
        let field_with_quote = csv_field(r#"Empresa "XYZ""#);
        // Debe encerrar en comillas y escapar las internas
        assert!(field_with_quote.starts_with('"'));
        assert!(field_with_quote.contains("\"\""));
    }

    #[test]
    fn test_export_sales_book_totals() {
        let entries = vec![
            sample_sales_entry(),
            SalesBookExportEntry {
                entry_date: NaiveDate::from_ymd_opt(2026, 3, 20).unwrap(),
                buyer_name: "Otro Cliente".to_string(),
                buyer_rif: Some("J-11111111-1".to_string()),
                invoice_number: "00000002".to_string(),
                control_number: "00-00000002".to_string(),
                total_amount: dec!(116.00),
                exempt_base: dec!(0.00),
                general_base: dec!(100.00),
                general_tax: dec!(16.00),
                reduced_base: dec!(0.00),
                reduced_tax: dec!(0.00),
                is_summary: false,
            },
        ];

        let data = SalesBookExportData {
            period: "2026-03".to_string(),
            entries,
        };

        let csv = String::from_utf8(export_sales_book(&data).unwrap()).unwrap();
        assert!(csv.contains("TOTALES"));
        assert!(csv.contains("696.00")); // 580 + 116
    }
}
