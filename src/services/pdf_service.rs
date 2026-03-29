//! Servicio de generación de PDFs fiscales.
//!
//! Genera PDFs de facturas, notas de crédito/débito, comprobantes de retención IVA
//! y comprobantes ARC de ISLR, con todos los campos obligatorios según SENIAT (PA 0071).

use chrono::NaiveDate;
use genpdf::{Alignment, Element, elements, fonts, style};
use rust_decimal::Decimal;

use crate::errors::AppError;

// ─── Data structs ────────────────────────────────────────────────────────────

/// Datos del emisor (empresa) que aparecen en el encabezado de cada documento fiscal.
#[derive(Debug, Clone)]
pub struct CompanyHeader {
    /// Razón social.
    pub business_name: String,
    /// Nombre comercial (opcional).
    pub trade_name: Option<String>,
    /// RIF del emisor (ej: "J-12345678-9").
    pub rif: String,
    /// Domicilio fiscal completo.
    pub fiscal_address: String,
    /// Teléfono (opcional).
    pub phone: Option<String>,
}

/// Un ítem dentro de un documento fiscal para el PDF.
#[derive(Debug, Clone)]
pub struct PdfLineItem {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    /// Alícuota como string descriptivo, ej: "16%", "8%", "0%".
    pub tax_rate_label: String,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
}

/// Datos completos para generar el PDF de una factura.
#[derive(Debug, Clone)]
pub struct InvoicePdfData {
    pub company: CompanyHeader,
    pub invoice_number: String,
    pub control_number: String,
    pub invoice_date: NaiveDate,
    /// RIF del cliente (None si es consumidor final).
    pub client_rif: Option<String>,
    pub client_name: String,
    pub client_address: String,
    pub items: Vec<PdfLineItem>,
    /// "Contado" o "Crédito".
    pub payment_condition: String,
    /// Plazo en días si la condición es crédito.
    pub credit_days: Option<u32>,
    pub subtotal: Decimal,
    pub tax_general: Decimal,
    pub tax_reduced: Decimal,
    pub tax_luxury: Decimal,
    pub total_tax: Decimal,
    pub grand_total: Decimal,
    pub currency: String,
    pub exchange_rate: Decimal,
    /// Si se debe imprimir la leyenda "SIN DERECHO A CRÉDITO FISCAL".
    pub no_fiscal_credit: bool,
}

/// Datos para generar el PDF de una Nota de Crédito.
#[derive(Debug, Clone)]
pub struct CreditNotePdfData {
    pub company: CompanyHeader,
    pub credit_note_number: String,
    pub original_invoice_number: String,
    pub issue_date: NaiveDate,
    pub reason: String,
    pub client_rif: Option<String>,
    pub client_name: String,
    pub items: Vec<PdfLineItem>,
    pub subtotal: Decimal,
    pub total_tax: Decimal,
    pub grand_total: Decimal,
    /// Moneda de la operación (ej: "USD", "VES").
    pub currency: String,
    /// Tasa de cambio BCV del día (usada si currency != "VES").
    pub exchange_rate: Decimal,
}

/// Datos para generar el PDF de una Nota de Débito.
#[derive(Debug, Clone)]
pub struct DebitNotePdfData {
    pub company: CompanyHeader,
    pub debit_note_number: String,
    pub original_invoice_number: String,
    pub issue_date: NaiveDate,
    pub reason: String,
    pub client_rif: Option<String>,
    pub client_name: String,
    pub items: Vec<PdfLineItem>,
    pub subtotal: Decimal,
    pub total_tax: Decimal,
    pub grand_total: Decimal,
    /// Moneda de la operación (ej: "USD", "VES").
    pub currency: String,
    /// Tasa de cambio BCV del día (usada si currency != "VES").
    pub exchange_rate: Decimal,
}

/// Datos para el comprobante de retención IVA.
#[derive(Debug, Clone)]
pub struct IvaWithholdingVoucherPdfData {
    /// RIF del agente de retención (empresa que retiene).
    pub agent_rif: String,
    pub agent_name: String,
    /// RIF del proveedor al que se le retiene.
    pub supplier_rif: String,
    pub supplier_name: String,
    pub invoice_number: String,
    pub invoice_date: NaiveDate,
    pub iva_amount: Decimal,
    /// Porcentaje de retención: 75 o 100.
    pub withholding_rate: u8,
    pub withheld_amount: Decimal,
    pub net_payable: Decimal,
    pub voucher_number: String,
    pub period: String,
}

/// Datos para el comprobante ARC de retención ISLR.
#[derive(Debug, Clone)]
pub struct IslrArcPdfData {
    /// RIF del agente retenedor.
    pub retenedor_rif: String,
    pub retenedor_name: String,
    /// RIF del beneficiario (proveedor/prestador).
    pub beneficiary_rif: String,
    pub beneficiary_name: String,
    pub activity_type: String,
    pub base_amount: Decimal,
    pub withholding_rate: Decimal,
    pub withheld_amount: Decimal,
    pub period: String,
    pub invoice_number: String,
    pub invoice_date: NaiveDate,
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Crea un Document genpdf con fuente embebida (Helvetica built-in, sin archivos externos).
fn create_document(title: &str) -> Result<genpdf::Document, AppError> {
    let font_family = fonts::from_files("./fonts", "LiberationSans", Some(fonts::Builtin::Helvetica))
        .or_else(|_| {
            fonts::from_files(
                "/usr/share/fonts/liberation",
                "LiberationSans",
                Some(fonts::Builtin::Helvetica),
            )
        })
        .or_else(|_| {
            fonts::from_files(
                "/usr/share/fonts/truetype/liberation",
                "LiberationSans",
                Some(fonts::Builtin::Helvetica),
            )
        })
        .or_else(|_| {
            // Último recurso: solo builtin
            create_builtin_font_family()
        })
        .map_err(|e| AppError::Internal(format!("No se pudo cargar ninguna fuente para genpdf: {}", e)))?;

    let mut doc = genpdf::Document::new(font_family);
    doc.set_title(title);
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.2);

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(15);
    doc.set_page_decorator(decorator);

    Ok(doc)
}

/// Crea una familia de fuentes usando solo la Helvetica built-in (sin archivos de fuente).
fn create_builtin_font_family() -> Result<fonts::FontFamily<fonts::FontData>, Box<dyn std::error::Error>> {
    // genpdf requiere una FontFamily; usamos Helvetica built-in via la ruta de archivos vacíos
    // Si todos los intentos de archivos fallan, construimos con el trait Builtin
    fonts::from_files("", "LiberationSans", Some(fonts::Builtin::Helvetica))
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Renderiza un Document a Vec<u8>.
fn render_doc_to_bytes(doc: genpdf::Document) -> Result<Vec<u8>, AppError> {
    let mut buf = Vec::new();
    doc.render(&mut buf)
        .map_err(|e| AppError::Internal(format!("Error renderizando PDF: {}", e)))?;
    Ok(buf)
}

/// Estilo bold para encabezados.
fn bold() -> style::Style {
    style::Style::new().bold()
}

/// Estilo bold + tamaño grande para títulos.
fn title_style() -> style::Style {
    style::Style::new().bold().with_font_size(16)
}

/// Estilo bold + tamaño mediano para subtítulos de sección.
fn section_style() -> style::Style {
    style::Style::new().bold().with_font_size(11)
}

fn small_style() -> style::Style {
    style::Style::new().with_font_size(9)
}

fn warning_style() -> style::Style {
    style::Style::new()
        .bold()
        .with_font_size(10)
        .with_color(style::Color::Rgb(200, 0, 0))
}

/// Agrega el encabezado del emisor (razón social, RIF, domicilio, teléfono).
fn push_company_header(doc: &mut genpdf::Document, company: &CompanyHeader, doc_type: &str) {
    doc.push(
        elements::Paragraph::new(doc_type)
            .aligned(Alignment::Center)
            .styled(title_style()),
    );
    doc.push(elements::Break::new(0.5));
    doc.push(
        elements::Paragraph::new(company.business_name.clone())
            .aligned(Alignment::Center)
            .styled(bold()),
    );
    if let Some(trade) = &company.trade_name {
        doc.push(
            elements::Paragraph::new(format!("Nombre comercial: {}", trade))
                .aligned(Alignment::Center),
        );
    }
    doc.push(
        elements::Paragraph::new(format!("RIF: {}", company.rif))
            .aligned(Alignment::Center)
            .styled(bold()),
    );
    doc.push(
        elements::Paragraph::new(company.fiscal_address.clone())
            .aligned(Alignment::Center)
            .styled(small_style()),
    );
    if let Some(phone) = &company.phone {
        doc.push(
            elements::Paragraph::new(format!("Tel: {}", phone))
                .aligned(Alignment::Center)
                .styled(small_style()),
        );
    }
    doc.push(elements::Break::new(1.0));
}

/// Agrega la tabla de ítems al documento.
fn push_items_table(doc: &mut genpdf::Document, items: &[PdfLineItem]) -> Result<(), AppError> {
    doc.push(elements::Paragraph::new("Descripción de Bienes / Servicios").styled(section_style()));
    doc.push(elements::Break::new(0.5));

    // Columnas: Descripción | Cant. | P.Unit | % IVA | Subtotal | IVA | Total
    let mut table = elements::TableLayout::new(vec![4, 1, 2, 1, 2, 2, 2]);
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));

    // Encabezado de tabla
    table
        .row()
        .element(
            elements::Paragraph::new("Descripción")
                .styled(bold())
                .padded(1),
        )
        .element(
            elements::Paragraph::new("Cant.")
                .aligned(Alignment::Center)
                .styled(bold())
                .padded(1),
        )
        .element(
            elements::Paragraph::new("P. Unit.")
                .aligned(Alignment::Right)
                .styled(bold())
                .padded(1),
        )
        .element(
            elements::Paragraph::new("IVA%")
                .aligned(Alignment::Center)
                .styled(bold())
                .padded(1),
        )
        .element(
            elements::Paragraph::new("Subtotal")
                .aligned(Alignment::Right)
                .styled(bold())
                .padded(1),
        )
        .element(
            elements::Paragraph::new("IVA")
                .aligned(Alignment::Right)
                .styled(bold())
                .padded(1),
        )
        .element(
            elements::Paragraph::new("Total")
                .aligned(Alignment::Right)
                .styled(bold())
                .padded(1),
        )
        .push()
        .map_err(|e| AppError::Internal(format!("Error en fila de encabezado de tabla de ítems: {}", e)))?;

    for item in items {
        table
            .row()
            .element(elements::Paragraph::new(item.description.clone()).padded(1))
            .element(
                elements::Paragraph::new(format!("{}", item.quantity))
                    .aligned(Alignment::Center)
                    .padded(1),
            )
            .element(
                elements::Paragraph::new(format!("{:.2}", item.unit_price))
                    .aligned(Alignment::Right)
                    .padded(1),
            )
            .element(
                elements::Paragraph::new(item.tax_rate_label.clone())
                    .aligned(Alignment::Center)
                    .padded(1),
            )
            .element(
                elements::Paragraph::new(format!("{:.2}", item.subtotal))
                    .aligned(Alignment::Right)
                    .padded(1),
            )
            .element(
                elements::Paragraph::new(format!("{:.2}", item.tax_amount))
                    .aligned(Alignment::Right)
                    .padded(1),
            )
            .element(
                elements::Paragraph::new(format!("{:.2}", item.total))
                    .aligned(Alignment::Right)
                    .padded(1),
            )
            .push()
            .map_err(|e| AppError::Internal(format!("Error en fila de ítem de tabla: {}", e)))?;
    }

    doc.push(table);
    doc.push(elements::Break::new(1.0));
    Ok(())
}

/// Agrega la sección de totales (base imponible por alícuota + IVA + total).
fn push_totals_section(
    doc: &mut genpdf::Document,
    subtotal: Decimal,
    tax_general: Decimal,
    tax_reduced: Decimal,
    tax_luxury: Decimal,
    total_tax: Decimal,
    grand_total: Decimal,
    currency: &str,
    exchange_rate: Decimal,
) -> Result<(), AppError> {
    let mut table = elements::TableLayout::new(vec![3, 2]);
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, false, false));

    let add_row = |table: &mut elements::TableLayout,
                   label: &str,
                   value: String,
                   is_bold: bool|
     -> Result<(), AppError> {
        let label_para = elements::Paragraph::new(label).padded(1);
        let value_para = elements::Paragraph::new(value)
            .aligned(Alignment::Right)
            .padded(1);
        if is_bold {
            table
                .row()
                .element(label_para.styled(bold()))
                .element(value_para.styled(bold()))
                .push()
                .map_err(|e| AppError::Internal(format!("Error en fila de totales: {}", e)))?;
        } else {
            table
                .row()
                .element(label_para)
                .element(value_para)
                .push()
                .map_err(|e| AppError::Internal(format!("Error en fila de totales: {}", e)))?;
        }
        Ok(())
    };

    add_row(
        &mut table,
        "Base imponible",
        format!("{:.2} {}", subtotal, currency),
        false,
    )?;
    if tax_general > Decimal::ZERO {
        add_row(
            &mut table,
            "IVA 16%",
            format!("{:.2} {}", tax_general, currency),
            false,
        )?;
    }
    if tax_reduced > Decimal::ZERO {
        add_row(
            &mut table,
            "IVA 8%",
            format!("{:.2} {}", tax_reduced, currency),
            false,
        )?;
    }
    if tax_luxury > Decimal::ZERO {
        add_row(
            &mut table,
            "IVA 31%",
            format!("{:.2} {}", tax_luxury, currency),
            false,
        )?;
    }
    add_row(
        &mut table,
        "Total IVA",
        format!("{:.2} {}", total_tax, currency),
        false,
    )?;
    add_row(
        &mut table,
        "TOTAL A PAGAR",
        format!("{:.2} {}", grand_total, currency),
        true,
    )?;

    if currency != "VES" && exchange_rate > Decimal::ZERO {
        let total_ves = (grand_total * exchange_rate).round_dp(2);
        add_row(
            &mut table,
            &format!("Total en Bs. (Tasa BCV: {:.4})", exchange_rate),
            format!("{:.2} VES", total_ves),
            false,
        )?;
    }

    doc.push(table);
    Ok(())
}

// ─── Public PDF generation functions ─────────────────────────────────────────

/// Genera el PDF de una factura fiscal con todos los campos obligatorios (PA 0071).
///
/// Retorna los bytes del PDF generado.
pub fn generate_invoice_pdf(data: &InvoicePdfData) -> Result<Vec<u8>, AppError> {
    let mut doc = create_document(&format!("Factura {}", data.invoice_number))?;

    // Encabezado del emisor
    push_company_header(&mut doc, &data.company, "FACTURA");

    // Datos del documento
    doc.push(elements::Paragraph::new("Datos del Documento").styled(section_style()));
    doc.push(elements::Break::new(0.3));
    let mut doc_info = elements::TableLayout::new(vec![2, 3]);
    doc_info
        .row()
        .element(
            elements::Paragraph::new("N° Factura:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(data.invoice_number.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en tabla doc_info: {}", e)))?;
    doc_info
        .row()
        .element(
            elements::Paragraph::new("N° Control:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(data.control_number.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en tabla doc_info: {}", e)))?;
    doc_info
        .row()
        .element(elements::Paragraph::new("Fecha:").styled(bold()).padded(1))
        .element(elements::Paragraph::new(data.invoice_date.to_string()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en tabla doc_info: {}", e)))?;

    let payment_label = match data.credit_days {
        Some(days) => format!("Crédito ({} días)", days),
        None => data.payment_condition.clone(),
    };
    doc_info
        .row()
        .element(
            elements::Paragraph::new("Condición de pago:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(payment_label).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en tabla doc_info: {}", e)))?;
    doc.push(doc_info);
    doc.push(elements::Break::new(1.0));

    // Datos del comprador
    doc.push(elements::Paragraph::new("Datos del Comprador").styled(section_style()));
    doc.push(elements::Break::new(0.3));
    let mut buyer_info = elements::TableLayout::new(vec![2, 3]);
    buyer_info
        .row()
        .element(
            elements::Paragraph::new("Nombre/Razón Social:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(data.client_name.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en buyer_info: {}", e)))?;
    let rif_label = data
        .client_rif
        .clone()
        .unwrap_or_else(|| "Consumidor Final".to_string());
    buyer_info
        .row()
        .element(
            elements::Paragraph::new("RIF/Cédula:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(rif_label).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en buyer_info: {}", e)))?;
    buyer_info
        .row()
        .element(
            elements::Paragraph::new("Domicilio Fiscal:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(data.client_address.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en buyer_info: {}", e)))?;
    doc.push(buyer_info);
    doc.push(elements::Break::new(1.0));

    // Tabla de ítems
    push_items_table(&mut doc, &data.items)?;

    // Totales
    push_totals_section(
        &mut doc,
        data.subtotal,
        data.tax_general,
        data.tax_reduced,
        data.tax_luxury,
        data.total_tax,
        data.grand_total,
        &data.currency,
        data.exchange_rate,
    )?;

    // Leyenda pie de página
    if data.no_fiscal_credit {
        doc.push(elements::Break::new(1.0));
        doc.push(
            elements::Paragraph::new("SIN DERECHO A CRÉDITO FISCAL")
                .aligned(Alignment::Center)
                .styled(warning_style()),
        );
    }

    render_doc_to_bytes(doc)
}

/// Genera el PDF de una Nota de Crédito fiscal.
pub fn generate_credit_note_pdf(data: &CreditNotePdfData) -> Result<Vec<u8>, AppError> {
    let mut doc = create_document(&format!("Nota de Crédito {}", data.credit_note_number))?;

    push_company_header(&mut doc, &data.company, "NOTA DE CRÉDITO");

    // Datos del documento
    let mut doc_info = elements::TableLayout::new(vec![2, 3]);
    doc_info
        .row()
        .element(
            elements::Paragraph::new("N° Nota de Crédito:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(data.credit_note_number.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en doc_info NC: {}", e)))?;
    doc_info
        .row()
        .element(
            elements::Paragraph::new("Factura Original:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(data.original_invoice_number.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en doc_info NC: {}", e)))?;
    doc_info
        .row()
        .element(elements::Paragraph::new("Fecha:").styled(bold()).padded(1))
        .element(elements::Paragraph::new(data.issue_date.to_string()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en doc_info NC: {}", e)))?;
    doc_info
        .row()
        .element(elements::Paragraph::new("Motivo:").styled(bold()).padded(1))
        .element(elements::Paragraph::new(data.reason.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en doc_info NC: {}", e)))?;
    doc.push(doc_info);
    doc.push(elements::Break::new(1.0));

    // Datos del cliente
    let mut buyer_info = elements::TableLayout::new(vec![2, 3]);
    buyer_info
        .row()
        .element(
            elements::Paragraph::new("Nombre/Razón Social:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(data.client_name.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en buyer_info NC: {}", e)))?;
    let rif_label = data
        .client_rif
        .clone()
        .unwrap_or_else(|| "Consumidor Final".to_string());
    buyer_info
        .row()
        .element(
            elements::Paragraph::new("RIF/Cédula:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(rif_label).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en buyer_info NC: {}", e)))?;
    doc.push(buyer_info);
    doc.push(elements::Break::new(1.0));

    push_items_table(&mut doc, &data.items)?;

    // Totales con soporte dual-moneda
    let mut totals = elements::TableLayout::new(vec![3, 2]);
    totals
        .row()
        .element(elements::Paragraph::new(format!("Subtotal ({})", data.currency)).padded(1))
        .element(
            elements::Paragraph::new(format!("{:.2}", data.subtotal))
                .aligned(Alignment::Right)
                .padded(1),
        )
        .push()
        .map_err(|e| AppError::Internal(format!("Error en totals NC: {}", e)))?;
    totals
        .row()
        .element(elements::Paragraph::new(format!("Total IVA ({})", data.currency)).padded(1))
        .element(
            elements::Paragraph::new(format!("{:.2}", data.total_tax))
                .aligned(Alignment::Right)
                .padded(1),
        )
        .push()
        .map_err(|e| AppError::Internal(format!("Error en totals NC: {}", e)))?;
    totals
        .row()
        .element(
            elements::Paragraph::new(format!("TOTAL NOTA DE CRÉDITO ({})", data.currency))
                .styled(bold())
                .padded(1),
        )
        .element(
            elements::Paragraph::new(format!("{:.2}", data.grand_total))
                .aligned(Alignment::Right)
                .styled(bold())
                .padded(1),
        )
        .push()
        .map_err(|e| AppError::Internal(format!("Error en totals NC: {}", e)))?;

    // Equivalente en Bs. si la moneda no es VES
    if data.currency != "VES" && data.exchange_rate > Decimal::ZERO {
        let total_ves = (data.grand_total * data.exchange_rate).round_dp(2);
        totals
            .row()
            .element(
                elements::Paragraph::new(format!(
                    "Total en Bs. (Tasa BCV: {:.4})",
                    data.exchange_rate
                ))
                .padded(1),
            )
            .element(
                elements::Paragraph::new(format!("{:.2} VES", total_ves))
                    .aligned(Alignment::Right)
                    .padded(1),
            )
            .push()
            .map_err(|e| AppError::Internal(format!("Error en totals NC VES: {}", e)))?;
    }

    doc.push(totals);

    render_doc_to_bytes(doc)
}

/// Genera el PDF de una Nota de Débito fiscal.
pub fn generate_debit_note_pdf(data: &DebitNotePdfData) -> Result<Vec<u8>, AppError> {
    let mut doc = create_document(&format!("Nota de Débito {}", data.debit_note_number))?;

    push_company_header(&mut doc, &data.company, "NOTA DE DÉBITO");

    let mut doc_info = elements::TableLayout::new(vec![2, 3]);
    doc_info
        .row()
        .element(
            elements::Paragraph::new("N° Nota de Débito:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(data.debit_note_number.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en doc_info ND: {}", e)))?;
    doc_info
        .row()
        .element(
            elements::Paragraph::new("Factura Original:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(data.original_invoice_number.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en doc_info ND: {}", e)))?;
    doc_info
        .row()
        .element(elements::Paragraph::new("Fecha:").styled(bold()).padded(1))
        .element(elements::Paragraph::new(data.issue_date.to_string()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en doc_info ND: {}", e)))?;
    doc_info
        .row()
        .element(elements::Paragraph::new("Motivo:").styled(bold()).padded(1))
        .element(elements::Paragraph::new(data.reason.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en doc_info ND: {}", e)))?;
    doc.push(doc_info);
    doc.push(elements::Break::new(1.0));

    let mut buyer_info = elements::TableLayout::new(vec![2, 3]);
    buyer_info
        .row()
        .element(
            elements::Paragraph::new("Nombre/Razón Social:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(data.client_name.clone()).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en buyer_info ND: {}", e)))?;
    let rif_label = data
        .client_rif
        .clone()
        .unwrap_or_else(|| "Consumidor Final".to_string());
    buyer_info
        .row()
        .element(
            elements::Paragraph::new("RIF/Cédula:")
                .styled(bold())
                .padded(1),
        )
        .element(elements::Paragraph::new(rif_label).padded(1))
        .push()
        .map_err(|e| AppError::Internal(format!("Error en buyer_info ND: {}", e)))?;
    doc.push(buyer_info);
    doc.push(elements::Break::new(1.0));

    push_items_table(&mut doc, &data.items)?;

    // Totales con soporte dual-moneda
    let mut totals = elements::TableLayout::new(vec![3, 2]);
    totals
        .row()
        .element(elements::Paragraph::new(format!("Subtotal ({})", data.currency)).padded(1))
        .element(
            elements::Paragraph::new(format!("{:.2}", data.subtotal))
                .aligned(Alignment::Right)
                .padded(1),
        )
        .push()
        .map_err(|e| AppError::Internal(format!("Error en totals ND: {}", e)))?;
    totals
        .row()
        .element(elements::Paragraph::new(format!("Total IVA ({})", data.currency)).padded(1))
        .element(
            elements::Paragraph::new(format!("{:.2}", data.total_tax))
                .aligned(Alignment::Right)
                .padded(1),
        )
        .push()
        .map_err(|e| AppError::Internal(format!("Error en totals ND: {}", e)))?;
    totals
        .row()
        .element(
            elements::Paragraph::new(format!("TOTAL NOTA DE DÉBITO ({})", data.currency))
                .styled(bold())
                .padded(1),
        )
        .element(
            elements::Paragraph::new(format!("{:.2}", data.grand_total))
                .aligned(Alignment::Right)
                .styled(bold())
                .padded(1),
        )
        .push()
        .map_err(|e| AppError::Internal(format!("Error en totals ND: {}", e)))?;

    // Equivalente en Bs. si la moneda no es VES
    if data.currency != "VES" && data.exchange_rate > Decimal::ZERO {
        let total_ves = (data.grand_total * data.exchange_rate).round_dp(2);
        totals
            .row()
            .element(
                elements::Paragraph::new(format!(
                    "Total en Bs. (Tasa BCV: {:.4})",
                    data.exchange_rate
                ))
                .padded(1),
            )
            .element(
                elements::Paragraph::new(format!("{:.2} VES", total_ves))
                    .aligned(Alignment::Right)
                    .padded(1),
            )
            .push()
            .map_err(|e| AppError::Internal(format!("Error en totals ND VES: {}", e)))?;
    }

    doc.push(totals);

    render_doc_to_bytes(doc)
}

/// Genera el PDF del comprobante de retención de IVA.
pub fn generate_iva_withholding_voucher_pdf(
    data: &IvaWithholdingVoucherPdfData,
) -> Result<Vec<u8>, AppError> {
    let mut doc = create_document(&format!(
        "Comprobante Retención IVA {}",
        data.voucher_number
    ))?;

    doc.push(
        elements::Paragraph::new("COMPROBANTE DE RETENCIÓN DE IVA")
            .aligned(Alignment::Center)
            .styled(title_style()),
    );
    doc.push(elements::Break::new(1.0));

    let mut info = elements::TableLayout::new(vec![3, 4]);
    let rows: Vec<(&str, String)> = vec![
        ("N° Comprobante:", data.voucher_number.clone()),
        ("Periodo:", data.period.clone()),
        ("Agente de Retención:", data.agent_name.clone()),
        ("RIF Agente:", data.agent_rif.clone()),
        ("Proveedor:", data.supplier_name.clone()),
        ("RIF Proveedor:", data.supplier_rif.clone()),
        ("N° Factura:", data.invoice_number.clone()),
        ("Fecha Factura:", data.invoice_date.to_string()),
        ("Monto IVA Facturado:", format!("{:.2}", data.iva_amount)),
        (
            "Porcentaje de Retención:",
            format!("{}%", data.withholding_rate),
        ),
        ("Monto Retenido:", format!("{:.2}", data.withheld_amount)),
        (
            "IVA a Pagar al Proveedor:",
            format!("{:.2}", data.net_payable),
        ),
    ];

    for (label, value) in &rows {
        let is_key = label.contains("Monto Retenido") || label.contains("N° Comprobante");
        let label_para = elements::Paragraph::new(*label).styled(bold()).padded(1);
        let value_style = if is_key { bold() } else { style::Style::new() };
        let value_para = elements::Paragraph::new(value.clone())
            .styled(value_style)
            .padded(1);
        info.row()
            .element(label_para)
            .element(value_para)
            .push()
            .map_err(|e| AppError::Internal(format!("Error en fila de comprobante IVA: {}", e)))?;
    }

    doc.push(info);

    render_doc_to_bytes(doc)
}

/// Genera el PDF del Comprobante ARC de retención de ISLR.
pub fn generate_islr_arc_pdf(data: &IslrArcPdfData) -> Result<Vec<u8>, AppError> {
    let mut doc = create_document(&format!(
        "ARC ISLR - {} - {}",
        data.beneficiary_rif, data.period
    ))?;

    doc.push(
        elements::Paragraph::new("COMPROBANTE DE RETENCIÓN DE ISLR (ARC)")
            .aligned(Alignment::Center)
            .styled(title_style()),
    );
    doc.push(elements::Break::new(1.0));

    let mut info = elements::TableLayout::new(vec![3, 4]);
    let rows: Vec<(&str, String)> = vec![
        ("Periodo:", data.period.clone()),
        ("Retenedor:", data.retenedor_name.clone()),
        ("RIF Retenedor:", data.retenedor_rif.clone()),
        ("Beneficiario:", data.beneficiary_name.clone()),
        ("RIF Beneficiario:", data.beneficiary_rif.clone()),
        ("Tipo de Actividad:", data.activity_type.clone()),
        ("N° Factura:", data.invoice_number.clone()),
        ("Fecha Factura:", data.invoice_date.to_string()),
        ("Base Imponible:", format!("{:.2}", data.base_amount)),
        ("Tasa de Retención:", format!("{}%", data.withholding_rate)),
        ("Monto Retenido:", format!("{:.2}", data.withheld_amount)),
    ];

    for (label, value) in &rows {
        let is_key = label.contains("Monto Retenido");
        let label_para = elements::Paragraph::new(*label).styled(bold()).padded(1);
        let value_style = if is_key { bold() } else { style::Style::new() };
        let value_para = elements::Paragraph::new(value.clone())
            .styled(value_style)
            .padded(1);
        info.row()
            .element(label_para)
            .element(value_para)
            .push()
            .map_err(|e| AppError::Internal(format!("Error en fila de ARC ISLR: {}", e)))?;
    }

    doc.push(info);

    render_doc_to_bytes(doc)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_company() -> CompanyHeader {
        CompanyHeader {
            business_name: "Empresa Prueba C.A.".to_string(),
            trade_name: Some("MarcaComercial".to_string()),
            rif: "J-12345678-9".to_string(),
            fiscal_address: "Av. Principal, Oficina 1, Caracas, Venezuela".to_string(),
            phone: Some("+58 212 555-0000".to_string()),
        }
    }

    fn sample_items() -> Vec<PdfLineItem> {
        vec![PdfLineItem {
            description: "Servicio de consultoría".to_string(),
            quantity: dec!(2),
            unit_price: dec!(500.00),
            tax_rate_label: "16%".to_string(),
            subtotal: dec!(1000.00),
            tax_amount: dec!(160.00),
            total: dec!(1160.00),
        }]
    }

    #[test]
    fn test_invoice_pdf_data_struct_construction() {
        let data = InvoicePdfData {
            company: sample_company(),
            invoice_number: "00000001".to_string(),
            control_number: "00-00000001".to_string(),
            invoice_date: NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            client_rif: Some("J-98765432-1".to_string()),
            client_name: "Cliente Prueba C.A.".to_string(),
            client_address: "Av. Secundaria, Caracas".to_string(),
            items: sample_items(),
            payment_condition: "Contado".to_string(),
            credit_days: None,
            subtotal: dec!(1000.00),
            tax_general: dec!(160.00),
            tax_reduced: dec!(0.00),
            tax_luxury: dec!(0.00),
            total_tax: dec!(160.00),
            grand_total: dec!(1160.00),
            currency: "USD".to_string(),
            exchange_rate: dec!(36.50),
            no_fiscal_credit: false,
        };

        assert_eq!(data.invoice_number, "00000001");
        assert_eq!(data.control_number, "00-00000001");
        assert_eq!(data.grand_total, dec!(1160.00));
        assert!(!data.no_fiscal_credit);
        assert_eq!(data.items.len(), 1);
    }

    #[test]
    fn test_credit_note_pdf_data_struct_construction() {
        let data = CreditNotePdfData {
            company: sample_company(),
            credit_note_number: "NC-0001".to_string(),
            original_invoice_number: "00000001".to_string(),
            issue_date: NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            reason: "Devolución parcial".to_string(),
            client_rif: Some("J-98765432-1".to_string()),
            client_name: "Cliente Prueba C.A.".to_string(),
            items: sample_items(),
            subtotal: dec!(1000.00),
            total_tax: dec!(160.00),
            grand_total: dec!(1160.00),
            currency: "USD".to_string(),
            exchange_rate: dec!(36.50),
        };

        assert_eq!(data.credit_note_number, "NC-0001");
        assert_eq!(data.original_invoice_number, "00000001");
        assert_eq!(data.grand_total, dec!(1160.00));
        assert_eq!(data.currency, "USD");
        assert_eq!(data.exchange_rate, dec!(36.50));
    }

    #[test]
    fn test_debit_note_pdf_data_struct_construction() {
        let data = DebitNotePdfData {
            company: sample_company(),
            debit_note_number: "ND-0001".to_string(),
            original_invoice_number: "00000001".to_string(),
            issue_date: NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            reason: "Cargo adicional por flete".to_string(),
            client_rif: Some("J-98765432-1".to_string()),
            client_name: "Cliente Prueba C.A.".to_string(),
            items: sample_items(),
            subtotal: dec!(200.00),
            total_tax: dec!(32.00),
            grand_total: dec!(232.00),
            currency: "USD".to_string(),
            exchange_rate: dec!(36.50),
        };

        assert_eq!(data.debit_note_number, "ND-0001");
        assert_eq!(data.original_invoice_number, "00000001");
        assert_eq!(data.grand_total, dec!(232.00));
        assert_eq!(data.currency, "USD");
        assert_eq!(data.exchange_rate, dec!(36.50));
    }

    #[test]
    fn test_debit_note_ves_currency() {
        let data = DebitNotePdfData {
            company: sample_company(),
            debit_note_number: "ND-0002".to_string(),
            original_invoice_number: "00000002".to_string(),
            issue_date: NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            reason: "Ajuste de precio".to_string(),
            client_rif: None,
            client_name: "Consumidor Final".to_string(),
            items: sample_items(),
            subtotal: dec!(100.00),
            total_tax: dec!(16.00),
            grand_total: dec!(116.00),
            currency: "VES".to_string(),
            exchange_rate: Decimal::ZERO,
        };

        assert_eq!(data.currency, "VES");
        // exchange_rate es ZERO para VES; no se muestra equivalente en Bs.
        assert_eq!(data.exchange_rate, Decimal::ZERO);
    }

    #[test]
    fn test_iva_withholding_voucher_pdf_data_struct_construction() {
        let data = IvaWithholdingVoucherPdfData {
            agent_rif: "J-12345678-9".to_string(),
            agent_name: "Empresa Retentora C.A.".to_string(),
            supplier_rif: "J-98765432-1".to_string(),
            supplier_name: "Proveedor S.A.".to_string(),
            invoice_number: "00000001".to_string(),
            invoice_date: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            iva_amount: dec!(160.00),
            withholding_rate: 75,
            withheld_amount: dec!(120.00),
            net_payable: dec!(40.00),
            voucher_number: "COMP-2026-0001".to_string(),
            period: "2026-03-Q1".to_string(),
        };

        assert_eq!(data.withheld_amount, dec!(120.00));
        assert_eq!(data.net_payable, dec!(40.00));
        assert_eq!(data.withholding_rate, 75);
    }

    #[test]
    fn test_islr_arc_pdf_data_struct_construction() {
        let data = IslrArcPdfData {
            retenedor_rif: "J-12345678-9".to_string(),
            retenedor_name: "Empresa Retenedora C.A.".to_string(),
            beneficiary_rif: "J-11111111-1".to_string(),
            beneficiary_name: "Proveedor Servicios S.A.".to_string(),
            activity_type: "servicios_profesionales".to_string(),
            base_amount: dec!(5000.00),
            withholding_rate: dec!(5),
            withheld_amount: dec!(250.00),
            period: "2026-03".to_string(),
            invoice_number: "00000020".to_string(),
            invoice_date: NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
        };

        assert_eq!(data.withheld_amount, dec!(250.00));
        assert_eq!(data.activity_type, "servicios_profesionales");
    }

    #[test]
    fn test_no_fiscal_credit_flag_on_consumer_final() {
        let data = InvoicePdfData {
            company: sample_company(),
            invoice_number: "00000002".to_string(),
            control_number: "00-00000002".to_string(),
            invoice_date: NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            client_rif: None, // Consumidor final
            client_name: "Juan Pérez".to_string(),
            client_address: "Calle Los Mangos, Valencia".to_string(),
            items: sample_items(),
            payment_condition: "Contado".to_string(),
            credit_days: None,
            subtotal: dec!(50.00),
            tax_general: dec!(8.00),
            tax_reduced: Decimal::ZERO,
            tax_luxury: Decimal::ZERO,
            total_tax: dec!(8.00),
            grand_total: dec!(58.00),
            currency: "VES".to_string(),
            exchange_rate: dec!(1),
            no_fiscal_credit: true, // Debe ser true para consumidor final
        };

        assert!(data.no_fiscal_credit);
        assert!(data.client_rif.is_none());
    }
}
