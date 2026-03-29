//! Servicio de generación de archivos XML para retenciones de IVA (SENIAT).
//!
//! Genera el archivo XML en formato esperado por el portal fiscal del SENIAT
//! para la declaración quincenal de retenciones de IVA.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::errors::AppError;

// ─── Data structs ─────────────────────────────────────────────────────────────

/// Datos de una retención individual para el XML.
#[derive(Debug, Clone)]
pub struct IvaWithholdingXmlData {
    /// RIF del proveedor.
    pub supplier_rif: String,
    /// Razón social del proveedor.
    pub supplier_name: String,
    /// Número de la factura retenida.
    pub invoice_number: String,
    /// Número de control de la factura.
    pub control_number: String,
    /// Fecha de la operación.
    pub operation_date: NaiveDate,
    /// Monto total facturado (incluyendo IVA).
    pub invoiced_amount: Decimal,
    /// Base imponible.
    pub taxable_base: Decimal,
    /// Monto del IVA en la factura.
    pub iva_amount: Decimal,
    /// Monto retenido.
    pub withheld_amount: Decimal,
    /// Porcentaje de retención: 75 o 100.
    pub withholding_percentage: u8,
}

// ─── XML generation ───────────────────────────────────────────────────────────

/// Genera el archivo XML de retenciones de IVA en formato SENIAT para declaración quincenal.
///
/// # Parámetros
/// - `withholdings`: lista de retenciones del periodo.
/// - `agent_rif`: RIF del agente de retención (ej: "J-12345678-9").
/// - `period`: periodo en formato "YYYYMM-QQ" (ej: "202603-01" para primera quincena de marzo 2026).
///
/// # Retorna
/// Los bytes UTF-8 del XML generado.
pub fn generate_iva_withholding_xml(
    withholdings: &[IvaWithholdingXmlData],
    agent_rif: &str,
    period: &str,
) -> Result<Vec<u8>, AppError> {
    let mut xml = String::new();

    // Declaración XML
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');

    // Elemento raíz
    xml.push_str(&format!(
        r#"<RelacionRetencionesIVA RifAgenteRetencion="{}" Periodo="{}">"#,
        escape_xml(agent_rif),
        escape_xml(period)
    ));
    xml.push('\n');

    // Un DetalleRetencion por cada retención
    for wh in withholdings {
        xml.push_str("  <DetalleRetencion>\n");
        xml.push_str(&format!(
            "    <RifProveedor>{}</RifProveedor>\n",
            escape_xml(&wh.supplier_rif)
        ));
        xml.push_str(&format!(
            "    <RazonSocialProveedor>{}</RazonSocialProveedor>\n",
            escape_xml(&wh.supplier_name)
        ));
        xml.push_str(&format!(
            "    <NumeroFactura>{}</NumeroFactura>\n",
            escape_xml(&wh.invoice_number)
        ));
        xml.push_str(&format!(
            "    <NumeroControl>{}</NumeroControl>\n",
            escape_xml(&wh.control_number)
        ));
        xml.push_str(&format!(
            "    <FechaOperacion>{}</FechaOperacion>\n",
            wh.operation_date
        ));
        xml.push_str(&format!(
            "    <MontoFacturado>{:.2}</MontoFacturado>\n",
            wh.invoiced_amount
        ));
        xml.push_str(&format!(
            "    <BaseImponible>{:.2}</BaseImponible>\n",
            wh.taxable_base
        ));
        xml.push_str(&format!("    <MontoIva>{:.2}</MontoIva>\n", wh.iva_amount));
        xml.push_str(&format!(
            "    <MontoRetenido>{:.2}</MontoRetenido>\n",
            wh.withheld_amount
        ));
        xml.push_str(&format!(
            "    <PorcentajeRetencion>{}</PorcentajeRetencion>\n",
            wh.withholding_percentage
        ));
        xml.push_str("  </DetalleRetencion>\n");
    }

    xml.push_str("</RelacionRetencionesIVA>\n");

    Ok(xml.into_bytes())
}

/// Escapa caracteres especiales XML para prevenir XML injection.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_withholding() -> IvaWithholdingXmlData {
        IvaWithholdingXmlData {
            supplier_rif: "V-12345678-9".to_string(),
            supplier_name: "Proveedor S.A.".to_string(),
            invoice_number: "00001234".to_string(),
            control_number: "00-00001234".to_string(),
            operation_date: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            invoiced_amount: dec!(1160.00),
            taxable_base: dec!(1000.00),
            iva_amount: dec!(160.00),
            withheld_amount: dec!(120.00),
            withholding_percentage: 75,
        }
    }

    #[test]
    fn test_generate_iva_withholding_xml_structure() {
        let withholdings = vec![sample_withholding()];
        let result = generate_iva_withholding_xml(&withholdings, "J-12345678-9", "202603-01");

        assert!(result.is_ok());
        let xml_bytes = result.unwrap();
        let xml = String::from_utf8(xml_bytes).expect("XML should be valid UTF-8");

        // Verificar declaración XML
        assert!(xml.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        // Verificar elemento raíz con atributos
        assert!(xml.contains(r#"RifAgenteRetencion="J-12345678-9""#));
        assert!(xml.contains(r#"Periodo="202603-01""#));
        assert!(xml.contains("<RelacionRetencionesIVA"));
        assert!(xml.contains("</RelacionRetencionesIVA>"));
    }

    #[test]
    fn test_generate_iva_withholding_xml_detail_fields() {
        let withholdings = vec![sample_withholding()];
        let result = generate_iva_withholding_xml(&withholdings, "J-12345678-9", "202603-01");

        let xml = String::from_utf8(result.unwrap()).unwrap();

        assert!(xml.contains("<DetalleRetencion>"));
        assert!(xml.contains("</DetalleRetencion>"));
        assert!(xml.contains("<RifProveedor>V-12345678-9</RifProveedor>"));
        assert!(xml.contains("<NumeroFactura>00001234</NumeroFactura>"));
        assert!(xml.contains("<NumeroControl>00-00001234</NumeroControl>"));
        assert!(xml.contains("<FechaOperacion>2026-03-15</FechaOperacion>"));
        assert!(xml.contains("<MontoFacturado>1160.00</MontoFacturado>"));
        assert!(xml.contains("<BaseImponible>1000.00</BaseImponible>"));
        assert!(xml.contains("<MontoIva>160.00</MontoIva>"));
        assert!(xml.contains("<MontoRetenido>120.00</MontoRetenido>"));
        assert!(xml.contains("<PorcentajeRetencion>75</PorcentajeRetencion>"));
    }

    #[test]
    fn test_generate_iva_withholding_xml_multiple_entries() {
        let w1 = IvaWithholdingXmlData {
            supplier_rif: "J-11111111-1".to_string(),
            supplier_name: "Proveedor Uno C.A.".to_string(),
            invoice_number: "00000100".to_string(),
            control_number: "00-00000100".to_string(),
            operation_date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            invoiced_amount: dec!(580.00),
            taxable_base: dec!(500.00),
            iva_amount: dec!(80.00),
            withheld_amount: dec!(80.00),
            withholding_percentage: 100,
        };
        let w2 = IvaWithholdingXmlData {
            supplier_rif: "J-22222222-2".to_string(),
            supplier_name: "Proveedor Dos S.A.".to_string(),
            invoice_number: "00000200".to_string(),
            control_number: "00-00000200".to_string(),
            operation_date: NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            invoiced_amount: dec!(232.00),
            taxable_base: dec!(200.00),
            iva_amount: dec!(32.00),
            withheld_amount: dec!(24.00),
            withholding_percentage: 75,
        };

        let result = generate_iva_withholding_xml(&[w1, w2], "J-99999999-9", "202603-01");
        let xml = String::from_utf8(result.unwrap()).unwrap();

        // Debe haber dos DetalleRetencion
        let count = xml.matches("<DetalleRetencion>").count();
        assert_eq!(
            count, 2,
            "Debe haber exactamente 2 entradas DetalleRetencion"
        );

        assert!(xml.contains("<RifProveedor>J-11111111-1</RifProveedor>"));
        assert!(xml.contains("<RifProveedor>J-22222222-2</RifProveedor>"));
    }

    #[test]
    fn test_generate_iva_withholding_xml_empty_list() {
        let result = generate_iva_withholding_xml(&[], "J-12345678-9", "202603-01");
        assert!(result.is_ok());
        let xml = String::from_utf8(result.unwrap()).unwrap();

        // Sin entries, el XML debe ser válido y sin DetalleRetencion
        assert!(xml.contains("<RelacionRetencionesIVA"));
        assert!(xml.contains("</RelacionRetencionesIVA>"));
        assert!(!xml.contains("<DetalleRetencion>"));
    }

    #[test]
    fn test_escape_xml_special_chars() {
        let result = escape_xml("Empresa & Cia. <test> \"value\" 'quote'");
        assert_eq!(
            result,
            "Empresa &amp; Cia. &lt;test&gt; &quot;value&quot; &apos;quote&apos;"
        );
    }

    #[test]
    fn test_xml_decimal_two_decimal_places() {
        let wh = IvaWithholdingXmlData {
            supplier_rif: "J-12345678-9".to_string(),
            supplier_name: "Test".to_string(),
            invoice_number: "00000001".to_string(),
            control_number: "00-00000001".to_string(),
            operation_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            invoiced_amount: dec!(1000),
            taxable_base: dec!(1000),
            iva_amount: dec!(160),
            withheld_amount: dec!(120),
            withholding_percentage: 75,
        };

        let xml = String::from_utf8(
            generate_iva_withholding_xml(&[wh], "J-12345678-9", "202601-01").unwrap(),
        )
        .unwrap();

        // Los montos deben tener exactamente 2 decimales
        assert!(xml.contains("<MontoIva>160.00</MontoIva>"));
        assert!(xml.contains("<MontoRetenido>120.00</MontoRetenido>"));
    }
}
