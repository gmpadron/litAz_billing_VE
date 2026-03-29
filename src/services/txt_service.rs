//! Servicio de generación de archivos TXT para retenciones de ISLR (SENIAT).
//!
//! Genera el archivo TXT en formato de campos posicionales esperado por el portal
//! fiscal del SENIAT para la declaración mensual de retenciones de ISLR.

use rust_decimal::Decimal;

use crate::errors::AppError;

// ─── Data structs ─────────────────────────────────────────────────────────────

/// Datos de una retención ISLR individual para el archivo TXT.
#[derive(Debug, Clone)]
pub struct IslrWithholdingTxtData {
    /// RIF del beneficiario (proveedor/prestador).
    pub beneficiary_rif: String,
    /// Razón social del beneficiario.
    pub beneficiary_name: String,
    /// Número de la factura asociada.
    pub invoice_number: String,
    /// Tipo de actividad (ej: "servicios_profesionales", "alquiler", "honorarios").
    pub activity_type: String,
    /// Monto pagado (base imponible).
    pub paid_amount: Decimal,
    /// Porcentaje de retención (ej: 5.00 para 5%).
    pub withholding_rate: Decimal,
    /// Monto retenido.
    pub withheld_amount: Decimal,
}

// ─── TXT generation ───────────────────────────────────────────────────────────

/// Genera el archivo TXT de retenciones de ISLR en formato SENIAT (campos separados por tabulador).
///
/// Formato por línea (tab-separated):
/// RIF_RETENEDOR | RIF_BENEFICIARIO | RAZON_SOCIAL | NUM_FACTURA | TIPO_ACTIVIDAD | MONTO_PAGADO | PORCENTAJE | MONTO_RETENIDO
///
/// # Parámetros
/// - `withholdings`: lista de retenciones del periodo.
/// - `retenedor_rif`: RIF del agente retenedor (ej: "J-12345678-9").
/// - `period`: periodo mensual en formato "YYYY-MM" (ej: "2026-03").
///
/// # Retorna
/// Los bytes UTF-8 del TXT generado.
pub fn generate_islr_withholding_txt(
    withholdings: &[IslrWithholdingTxtData],
    retenedor_rif: &str,
    period: &str,
) -> Result<Vec<u8>, AppError> {
    let mut lines: Vec<String> = Vec::new();

    // Línea de cabecera del archivo
    lines.push(format!(
        "# Retenciones ISLR | Retenedor: {} | Periodo: {}",
        retenedor_rif, period
    ));
    lines.push("# Columnas: RIF_RETENEDOR\tRIF_BENEFICIARIO\tRAZON_SOCIAL\tNUM_FACTURA\tTIPO_ACTIVIDAD\tMONTO_PAGADO\tPORCENTAJE_RETENCION\tMONTO_RETENIDO".to_string());

    for wh in withholdings {
        let line = format!(
            "{}\t{}\t{}\t{}\t{}\t{:.2}\t{:.2}\t{:.2}",
            sanitize_field(retenedor_rif),
            sanitize_field(&wh.beneficiary_rif),
            sanitize_field(&wh.beneficiary_name),
            sanitize_field(&wh.invoice_number),
            sanitize_field(&wh.activity_type),
            wh.paid_amount,
            wh.withholding_rate,
            wh.withheld_amount,
        );
        lines.push(line);
    }

    // Línea de totales al final
    if !withholdings.is_empty() {
        let total_paid: Decimal = withholdings.iter().map(|w| w.paid_amount).sum();
        let total_withheld: Decimal = withholdings.iter().map(|w| w.withheld_amount).sum();
        lines.push(String::new());
        lines.push(format!(
            "# TOTALES\tRegistros: {}\tTotal Pagado: {:.2}\tTotal Retenido: {:.2}",
            withholdings.len(),
            total_paid,
            total_withheld
        ));
    }

    let content = lines.join("\n") + "\n";
    Ok(content.into_bytes())
}

/// Limpia un campo para que no contenga tabs ni saltos de línea (que romperían el formato).
fn sanitize_field(s: &str) -> String {
    s.replace(['\t', '\n'], " ").replace('\r', "")
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_withholding() -> IslrWithholdingTxtData {
        IslrWithholdingTxtData {
            beneficiary_rif: "J-98765432-1".to_string(),
            beneficiary_name: "Proveedor Servicios C.A.".to_string(),
            invoice_number: "00000050".to_string(),
            activity_type: "servicios_profesionales".to_string(),
            paid_amount: dec!(5000.00),
            withholding_rate: dec!(5.00),
            withheld_amount: dec!(250.00),
        }
    }

    #[test]
    fn test_generate_islr_txt_structure() {
        let withholdings = vec![sample_withholding()];
        let result = generate_islr_withholding_txt(&withholdings, "J-12345678-9", "2026-03");

        assert!(result.is_ok());
        let txt = String::from_utf8(result.unwrap()).expect("TXT debe ser UTF-8 válido");

        // Debe tener cabecera
        assert!(txt.contains("Retenciones ISLR"));
        assert!(txt.contains("J-12345678-9"));
        assert!(txt.contains("2026-03"));
    }

    #[test]
    fn test_generate_islr_txt_data_fields() {
        let withholdings = vec![sample_withholding()];
        let result = generate_islr_withholding_txt(&withholdings, "J-12345678-9", "2026-03");

        let txt = String::from_utf8(result.unwrap()).unwrap();

        // Verificar que los datos de la retención aparecen en el archivo
        assert!(txt.contains("J-98765432-1"));
        assert!(txt.contains("Proveedor Servicios C.A."));
        assert!(txt.contains("00000050"));
        assert!(txt.contains("servicios_profesionales"));
        assert!(txt.contains("5000.00"));
        assert!(txt.contains("5.00"));
        assert!(txt.contains("250.00"));
    }

    #[test]
    fn test_generate_islr_txt_tab_separated() {
        let withholdings = vec![sample_withholding()];
        let result = generate_islr_withholding_txt(&withholdings, "J-12345678-9", "2026-03");

        let txt = String::from_utf8(result.unwrap()).unwrap();
        let lines: Vec<&str> = txt.lines().collect();

        // Las líneas de datos (no comentarios) deben tener 7 tabs = 8 campos
        let data_lines: Vec<&&str> = lines
            .iter()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .collect();
        assert_eq!(
            data_lines.len(),
            1,
            "Debe haber exactamente 1 línea de datos"
        );

        let fields: Vec<&str> = data_lines[0].split('\t').collect();
        assert_eq!(
            fields.len(),
            8,
            "Cada línea de datos debe tener 8 campos separados por tab"
        );
    }

    #[test]
    fn test_generate_islr_txt_multiple_entries() {
        let w1 = IslrWithholdingTxtData {
            beneficiary_rif: "J-11111111-1".to_string(),
            beneficiary_name: "Empresa Uno C.A.".to_string(),
            invoice_number: "00000010".to_string(),
            activity_type: "alquiler".to_string(),
            paid_amount: dec!(2000.00),
            withholding_rate: dec!(3.00),
            withheld_amount: dec!(60.00),
        };
        let w2 = sample_withholding();

        let result = generate_islr_withholding_txt(&[w1, w2], "J-12345678-9", "2026-03");
        let txt = String::from_utf8(result.unwrap()).unwrap();

        // Verificar que ambas retenciones están presentes
        assert!(txt.contains("J-11111111-1"));
        assert!(txt.contains("J-98765432-1"));
        assert!(txt.contains("alquiler"));
        assert!(txt.contains("servicios_profesionales"));
    }

    #[test]
    fn test_generate_islr_txt_totals_line() {
        let withholdings = vec![
            IslrWithholdingTxtData {
                beneficiary_rif: "J-11111111-1".to_string(),
                beneficiary_name: "Empresa Uno".to_string(),
                invoice_number: "001".to_string(),
                activity_type: "honorarios".to_string(),
                paid_amount: dec!(1000.00),
                withholding_rate: dec!(5.00),
                withheld_amount: dec!(50.00),
            },
            IslrWithholdingTxtData {
                beneficiary_rif: "J-22222222-2".to_string(),
                beneficiary_name: "Empresa Dos".to_string(),
                invoice_number: "002".to_string(),
                activity_type: "honorarios".to_string(),
                paid_amount: dec!(2000.00),
                withholding_rate: dec!(5.00),
                withheld_amount: dec!(100.00),
            },
        ];

        let result = generate_islr_withholding_txt(&withholdings, "J-99999999-9", "2026-01");
        let txt = String::from_utf8(result.unwrap()).unwrap();

        // Debe haber línea de totales
        assert!(txt.contains("TOTALES"));
        assert!(txt.contains("3000.00")); // Total pagado
        assert!(txt.contains("150.00")); // Total retenido
    }

    #[test]
    fn test_generate_islr_txt_empty_list() {
        let result = generate_islr_withholding_txt(&[], "J-12345678-9", "2026-03");
        assert!(result.is_ok());
        let txt = String::from_utf8(result.unwrap()).unwrap();

        // Sin retenciones, no debe haber línea de TOTALES (no tiene sentido)
        assert!(!txt.contains("TOTALES"));
        // Pero sí debe tener la cabecera
        assert!(txt.contains("Retenciones ISLR"));
    }

    #[test]
    fn test_sanitize_field_removes_tabs() {
        let dirty = "campo\tcon\ttabs";
        let clean = sanitize_field(dirty);
        assert!(!clean.contains('\t'));
        assert_eq!(clean, "campo con tabs");
    }

    #[test]
    fn test_decimal_amounts_two_decimal_places() {
        let wh = IslrWithholdingTxtData {
            beneficiary_rif: "J-12345678-9".to_string(),
            beneficiary_name: "Test".to_string(),
            invoice_number: "001".to_string(),
            activity_type: "servicios".to_string(),
            paid_amount: dec!(1000),
            withholding_rate: dec!(5),
            withheld_amount: dec!(50),
        };

        let txt = String::from_utf8(
            generate_islr_withholding_txt(&[wh], "J-99999999-9", "2026-01").unwrap(),
        )
        .unwrap();

        // Los montos enteros deben tener .00
        assert!(txt.contains("1000.00"));
        assert!(txt.contains("50.00"));
    }
}
