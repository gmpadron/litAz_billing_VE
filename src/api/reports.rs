//! Handlers HTTP para la generación y descarga de reportes fiscales.
//!
//! Capa delgada: extrae parámetros → llama al servicio de generación → devuelve bytes con
//! Content-Type apropiado (PDF, XML, CSV, TXT).

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::{ActiveCompanyId, AuthenticatedUser, require_billing_viewer};
use crate::services::{
    company_service,
    export_service::{
        PurchaseBookExportData, PurchaseBookExportEntry, SalesBookExportData, SalesBookExportEntry,
        export_purchase_book, export_sales_book,
    },
    txt_service::{IslrWithholdingTxtData, generate_islr_withholding_txt},
    xml_service::{IvaWithholdingXmlData, generate_iva_withholding_xml},
};

// ─── Query parameter structs ──────────────────────────────────────────────────

/// Parámetros de query para exportar retenciones IVA XML (periodo quincenal).
#[derive(Debug, Deserialize)]
pub struct IvaXmlQuery {
    /// Periodo en formato "YYYYMM-QQ" (ej: "202603-01" para primera quincena de marzo 2026,
    /// "202603-02" para la segunda quincena).
    pub period: String,
}

/// Parámetros de query para exportar retenciones ISLR TXT (periodo mensual).
#[derive(Debug, Deserialize)]
pub struct IslrTxtQuery {
    /// Periodo en formato "YYYY-MM" (ej: "2026-03").
    pub period: String,
}

/// Parámetros de query para exportar libros (periodo mensual).
#[derive(Debug, Deserialize)]
pub struct BookExportQuery {
    /// Periodo en formato "YYYY-MM" (ej: "2026-03").
    pub period: String,
}

// ─── PDF Handlers ─────────────────────────────────────────────────────────────

/// GET /billingVE/v1/reports/invoices/{id}/pdf
///
/// Descarga el PDF de una factura.
async fn get_invoice_pdf(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    path: web::Path<Uuid>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    use crate::services::invoice_service;
    use crate::services::pdf_service::{
        CompanyHeader, InvoicePdfData, PdfLineItem, generate_invoice_pdf,
    };

    let id = path.into_inner();

    let invoice = invoice_service::get_invoice(&db, id, company.0).await?;
    let company_profile = company_service::get_company_by_id(&db, company.0).await?;

    let company_header = CompanyHeader {
        business_name: company_profile.business_name.clone(),
        trade_name: company_profile.trade_name.clone(),
        rif: company_profile.rif.clone(),
        fiscal_address: company_profile.fiscal_address.clone(),
        phone: company_profile.phone.clone(),
    };

    let items: Vec<PdfLineItem> = invoice
        .items
        .iter()
        .map(|item| {
            let label = match item.tax_rate.as_str() {
                "general" => "16%",
                "reduced" => "8%",
                "luxury" => "31%",
                _ => "0%",
            };
            PdfLineItem {
                description: item.description.clone(),
                quantity: item.quantity,
                unit_price: item.unit_price,
                tax_rate_label: label.to_string(),
                subtotal: item.subtotal,
                tax_amount: item.tax_amount,
                total: item.total,
            }
        })
        .collect();

    let credit_days = invoice.credit_days;

    let data = InvoicePdfData {
        company: company_header,
        invoice_number: invoice.invoice_number.clone(),
        control_number: invoice.control_number.clone(),
        invoice_date: invoice.invoice_date,
        client_rif: invoice.client_rif.clone(),
        client_name: invoice.client_name.clone(),
        client_address: invoice.client_address.clone(),
        items,
        payment_condition: if invoice.payment_condition == "cash" {
            "Contado".to_string()
        } else {
            "Crédito".to_string()
        },
        credit_days,
        subtotal: invoice.subtotal,
        tax_general: invoice.tax_general,
        tax_reduced: invoice.tax_reduced,
        tax_luxury: invoice.tax_luxury,
        total_tax: invoice.total_tax,
        grand_total: invoice.grand_total,
        currency: invoice.currency.clone(),
        exchange_rate: invoice.exchange_rate,
        no_fiscal_credit: invoice.no_fiscal_credit,
    };

    let pdf_bytes = generate_invoice_pdf(&data)?;
    let filename = format!("factura_{}.pdf", invoice.invoice_number);

    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(pdf_bytes))
}

/// GET /billingVE/v1/reports/credit-notes/{id}/pdf
///
/// Descarga el PDF de una nota de crédito.
async fn get_credit_note_pdf(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    path: web::Path<Uuid>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    use crate::entities::credit_notes;
    use crate::services::credit_note_service;
    use crate::services::pdf_service::{
        CompanyHeader, CreditNotePdfData, PdfLineItem, generate_credit_note_pdf,
    };
    use rust_decimal::Decimal;
    use sea_orm::EntityTrait;

    let id = path.into_inner();

    let note = credit_note_service::get_credit_note(&db, id, company.0).await?;
    let company_profile = company_service::get_company_by_id(&db, company.0).await?;

    // Fetch the raw entity to get currency and exchange_rate_snapshot (not exposed by CreditNoteResponse)
    let note_entity = credit_notes::Entity::find_by_id(id)
        .one(db.get_ref())
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Nota de crédito con ID {} no encontrada", id)))?;

    let currency = note_entity.currency.clone();
    let exchange_rate = note_entity.exchange_rate_snapshot.unwrap_or(Decimal::ZERO);

    let company_header = CompanyHeader {
        business_name: company_profile.business_name.clone(),
        trade_name: company_profile.trade_name.clone(),
        rif: company_profile.rif.clone(),
        fiscal_address: company_profile.fiscal_address.clone(),
        phone: company_profile.phone.clone(),
    };

    let items: Vec<PdfLineItem> = note
        .items
        .iter()
        .map(|item| {
            let label = match item.tax_rate.as_str() {
                "general" => "16%",
                "reduced" => "8%",
                "luxury" => "31%",
                _ => "0%",
            };
            PdfLineItem {
                description: item.description.clone(),
                quantity: item.quantity,
                unit_price: item.unit_price,
                tax_rate_label: label.to_string(),
                subtotal: item.subtotal,
                tax_amount: item.tax_amount,
                total: item.total,
            }
        })
        .collect();

    let data = CreditNotePdfData {
        company: company_header,
        credit_note_number: note.credit_note_number.clone(),
        original_invoice_number: note.original_invoice_number.clone(),
        issue_date: note.issue_date,
        reason: note.reason.clone(),
        client_rif: note.client_rif.clone(),
        client_name: note.client_name.clone(),
        items,
        subtotal: note.subtotal,
        total_tax: note.total_tax,
        grand_total: note.grand_total,
        currency,
        exchange_rate,
    };

    let pdf_bytes = generate_credit_note_pdf(&data)?;
    let filename = format!("nota_credito_{}.pdf", note.credit_note_number);

    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(pdf_bytes))
}

/// GET /billingVE/v1/reports/debit-notes/{id}/pdf
///
/// Descarga el PDF de una nota de débito.
async fn get_debit_note_pdf(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    path: web::Path<Uuid>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    use crate::entities::debit_notes;
    use crate::services::debit_note_service;
    use crate::services::pdf_service::{
        CompanyHeader, DebitNotePdfData, PdfLineItem, generate_debit_note_pdf,
    };
    use rust_decimal::Decimal;
    use sea_orm::EntityTrait;

    let id = path.into_inner();

    let note = debit_note_service::get_debit_note(&db, id, company.0).await?;
    let company_profile = company_service::get_company_by_id(&db, company.0).await?;

    // Fetch the raw entity to get currency and exchange_rate_snapshot (not exposed by DebitNoteResponse)
    let note_entity = debit_notes::Entity::find_by_id(id)
        .one(db.get_ref())
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Nota de débito con ID {} no encontrada", id)))?;

    let currency = note_entity.currency.clone();
    let exchange_rate = note_entity.exchange_rate_snapshot.unwrap_or(Decimal::ZERO);

    let company_header = CompanyHeader {
        business_name: company_profile.business_name.clone(),
        trade_name: company_profile.trade_name.clone(),
        rif: company_profile.rif.clone(),
        fiscal_address: company_profile.fiscal_address.clone(),
        phone: company_profile.phone.clone(),
    };

    let items: Vec<PdfLineItem> = note
        .items
        .iter()
        .map(|item| {
            let label = match item.tax_rate.as_str() {
                "general" => "16%",
                "reduced" => "8%",
                "luxury" => "31%",
                _ => "0%",
            };
            PdfLineItem {
                description: item.description.clone(),
                quantity: item.quantity,
                unit_price: item.unit_price,
                tax_rate_label: label.to_string(),
                subtotal: item.subtotal,
                tax_amount: item.tax_amount,
                total: item.total,
            }
        })
        .collect();

    let data = DebitNotePdfData {
        company: company_header,
        debit_note_number: note.debit_note_number.clone(),
        original_invoice_number: note.original_invoice_number.clone(),
        issue_date: note.issue_date,
        reason: note.reason.clone(),
        client_rif: note.client_rif.clone(),
        client_name: note.client_name.clone(),
        items,
        subtotal: note.subtotal,
        total_tax: note.total_tax,
        grand_total: note.grand_total,
        currency,
        exchange_rate,
    };

    let pdf_bytes = generate_debit_note_pdf(&data)?;
    let filename = format!("nota_debito_{}.pdf", note.debit_note_number);

    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(pdf_bytes))
}

/// GET /billingVE/v1/reports/withholdings/iva/{id}/pdf
///
/// Descarga el comprobante de retención IVA en PDF.
async fn get_iva_withholding_voucher_pdf(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    path: web::Path<Uuid>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    use crate::services::invoice_service;
    use crate::services::pdf_service::{IvaWithholdingVoucherPdfData, generate_iva_withholding_voucher_pdf};
    use crate::services::withholding_iva_service;

    let id = path.into_inner();

    let wh = withholding_iva_service::get_iva_withholding(&db, id, company.0).await?;
    let company_profile = company_service::get_company_by_id(&db, company.0).await?;

    // Fetch the associated invoice to get its number and date
    let invoice = invoice_service::get_invoice(&db, wh.invoice_id, company.0).await?;

    let data = IvaWithholdingVoucherPdfData {
        agent_rif: company_profile.rif.clone(),
        agent_name: company_profile.business_name.clone(),
        supplier_rif: wh.supplier_rif.clone(),
        supplier_name: wh.supplier_name.clone(),
        invoice_number: invoice.invoice_number.clone(),
        invoice_date: invoice.invoice_date,
        iva_amount: wh.iva_amount,
        withholding_rate: wh.withholding_rate,
        withheld_amount: wh.withheld_amount,
        net_payable: wh.net_payable,
        voucher_number: wh.voucher_number.clone(),
        period: wh.period.clone(),
    };

    let pdf_bytes = generate_iva_withholding_voucher_pdf(&data)?;
    let filename = format!("comprobante_retencion_iva_{}.pdf", wh.voucher_number);

    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(pdf_bytes))
}

/// GET /billingVE/v1/reports/withholdings/islr/{id}/pdf
///
/// Descarga el comprobante ARC de retención ISLR en PDF.
async fn get_islr_arc_pdf(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    path: web::Path<Uuid>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    use crate::services::invoice_service;
    use crate::services::pdf_service::{IslrArcPdfData, generate_islr_arc_pdf};
    use crate::services::withholding_islr_service;

    let id = path.into_inner();

    let wh = withholding_islr_service::get_islr_withholding(&db, id, company.0).await?;
    let company_profile = company_service::get_company_by_id(&db, company.0).await?;

    // Fetch the associated invoice to get its number and date
    let invoice = invoice_service::get_invoice(&db, wh.invoice_id, company.0).await?;

    let data = IslrArcPdfData {
        retenedor_rif: company_profile.rif.clone(),
        retenedor_name: company_profile.business_name.clone(),
        beneficiary_rif: wh.beneficiary_rif.clone(),
        beneficiary_name: wh.beneficiary_name.clone(),
        activity_type: wh.activity_type.clone(),
        base_amount: wh.base_amount,
        withholding_rate: wh.withholding_rate,
        withheld_amount: wh.withheld_amount,
        period: wh.period.clone(),
        invoice_number: invoice.invoice_number.clone(),
        invoice_date: invoice.invoice_date,
    };

    let pdf_bytes = generate_islr_arc_pdf(&data)?;
    let filename = format!("arc_islr_{}.pdf", wh.id);

    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(pdf_bytes))
}

// ─── XML Handler ──────────────────────────────────────────────────────────────

/// GET /billingVE/v1/reports/withholdings/iva/xml?period=202603-01
///
/// Exporta las retenciones IVA del periodo como archivo XML para el SENIAT.
/// El periodo debe estar en formato "YYYYMM-QQ" (ej: "202603-01" para primera quincena,
/// "202603-02" para la segunda quincena de marzo 2026).
async fn get_iva_withholding_xml(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    query: web::Query<IvaXmlQuery>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    use crate::services::invoice_service;
    use crate::services::withholding_iva_service;

    let period = &query.period;

    if period.trim().is_empty() {
        return Err(AppError::BadRequest(
            "El parámetro 'period' es obligatorio (formato: YYYYMM-QQ, ej: 202603-01)".to_string(),
        ));
    }

    let company_profile = company_service::get_company_by_id(&db, company.0).await?;
    let agent_rif = company_profile.rif.clone();

    // Fetch withholdings for this period from the DB
    let period_withholdings =
        withholding_iva_service::get_iva_withholdings_for_period(&db, period, company.0).await?;

    // Build XML data structs, fetching invoice info for each withholding
    let mut xml_data: Vec<IvaWithholdingXmlData> = Vec::new();
    for wh in &period_withholdings {
        let invoice = invoice_service::get_invoice(&db, wh.invoice_id, company.0).await?;
        xml_data.push(IvaWithholdingXmlData {
            supplier_rif: wh.supplier_rif.clone(),
            supplier_name: wh.supplier_name.clone(),
            invoice_number: invoice.invoice_number.clone(),
            control_number: invoice.control_number.clone(),
            operation_date: invoice.invoice_date,
            invoiced_amount: invoice.grand_total,
            taxable_base: invoice.subtotal,
            iva_amount: wh.iva_amount,
            withheld_amount: wh.withheld_amount,
            withholding_percentage: wh.withholding_rate,
        });
    }

    let xml_bytes = generate_iva_withholding_xml(&xml_data, &agent_rif, period)?;

    let filename = format!("retenciones_iva_{}.xml", period.replace('-', "_"));

    Ok(HttpResponse::Ok()
        .content_type("application/xml; charset=utf-8")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(xml_bytes))
}

// ─── TXT Handler ──────────────────────────────────────────────────────────────

/// GET /billingVE/v1/reports/withholdings/islr/txt?period=2026-03
///
/// Exporta las retenciones ISLR del periodo como archivo TXT para el SENIAT.
async fn get_islr_withholding_txt(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    query: web::Query<IslrTxtQuery>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    use crate::services::invoice_service;
    use crate::services::withholding_islr_service;

    let period = &query.period;

    if period.trim().is_empty() {
        return Err(AppError::BadRequest(
            "El parámetro 'period' es obligatorio (formato: YYYY-MM)".to_string(),
        ));
    }

    let company_profile = company_service::get_company_by_id(&db, company.0).await?;
    let retenedor_rif = company_profile.rif.clone();

    // Fetch withholdings for this period from the DB
    let period_withholdings =
        withholding_islr_service::get_islr_withholdings_for_period(&db, period, company.0).await?;

    // Build TXT data structs, fetching invoice number for each withholding
    let mut txt_data: Vec<IslrWithholdingTxtData> = Vec::new();
    for wh in &period_withholdings {
        let invoice = invoice_service::get_invoice(&db, wh.invoice_id, company.0).await?;
        txt_data.push(IslrWithholdingTxtData {
            beneficiary_rif: wh.beneficiary_rif.clone(),
            beneficiary_name: wh.beneficiary_name.clone(),
            invoice_number: invoice.invoice_number.clone(),
            activity_type: wh.activity_type.clone(),
            paid_amount: wh.base_amount,
            withholding_rate: wh.withholding_rate,
            withheld_amount: wh.withheld_amount,
        });
    }

    let txt_bytes = generate_islr_withholding_txt(&txt_data, &retenedor_rif, period)?;

    let filename = format!("retenciones_islr_{}.txt", period.replace('-', "_"));

    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(txt_bytes))
}

// ─── Book Export Handlers ─────────────────────────────────────────────────────

/// GET /billingVE/v1/reports/books/purchases?period=2026-03
///
/// Exporta el Libro de Compras del periodo como CSV.
async fn get_purchase_book_export(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    query: web::Query<BookExportQuery>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    use crate::dto::book_dto::BookFilters;
    use crate::services::book_service;

    let period = &query.period;

    if period.trim().is_empty() {
        return Err(AppError::BadRequest(
            "El parámetro 'period' es obligatorio (formato: YYYY-MM)".to_string(),
        ));
    }

    let filters = BookFilters {
        period: Some(period.clone()),
        ..Default::default()
    };

    let book = book_service::get_purchase_book(&db, filters, company.0).await?;

    let entries: Vec<PurchaseBookExportEntry> = book
        .entries
        .into_iter()
        .map(|e| PurchaseBookExportEntry {
            entry_date: e.entry_date,
            supplier_name: e.supplier_name,
            supplier_rif: e.supplier_rif,
            invoice_number: e.invoice_number,
            control_number: e.control_number,
            total_amount: e.total_amount,
            exempt_base: e.exempt_base,
            general_base: e.general_base,
            general_tax: e.general_tax,
            reduced_base: e.reduced_base,
            reduced_tax: e.reduced_tax,
            iva_withheld: e.iva_withheld,
        })
        .collect();

    let data = PurchaseBookExportData {
        period: period.clone(),
        entries,
    };

    let csv_bytes = export_purchase_book(&data)?;

    let filename = format!("libro_compras_{}.csv", period.replace('-', "_"));

    Ok(HttpResponse::Ok()
        .content_type("text/csv; charset=utf-8")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(csv_bytes))
}

/// GET /billingVE/v1/reports/books/sales?period=2026-03
///
/// Exporta el Libro de Ventas del periodo como CSV.
async fn get_sales_book_export(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    query: web::Query<BookExportQuery>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    use crate::dto::book_dto::BookFilters;
    use crate::services::book_service;

    let period = &query.period;

    if period.trim().is_empty() {
        return Err(AppError::BadRequest(
            "El parámetro 'period' es obligatorio (formato: YYYY-MM)".to_string(),
        ));
    }

    let filters = BookFilters {
        period: Some(period.clone()),
        ..Default::default()
    };

    let book = book_service::get_sales_book(&db, filters, company.0).await?;

    let entries: Vec<SalesBookExportEntry> = book
        .entries
        .into_iter()
        .map(|e| SalesBookExportEntry {
            entry_date: e.entry_date,
            buyer_name: e.buyer_name,
            buyer_rif: e.buyer_rif,
            invoice_number: e.invoice_number,
            control_number: e.control_number,
            total_amount: e.total_amount,
            exempt_base: e.exempt_base,
            general_base: e.general_base,
            general_tax: e.general_tax,
            reduced_base: e.reduced_base,
            reduced_tax: e.reduced_tax,
            is_summary: e.is_summary,
        })
        .collect();

    let data = SalesBookExportData {
        period: period.clone(),
        entries,
    };

    let csv_bytes = export_sales_book(&data)?;

    let filename = format!("libro_ventas_{}.csv", period.replace('-', "_"));

    Ok(HttpResponse::Ok()
        .content_type("text/csv; charset=utf-8")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(csv_bytes))
}

// ─── Route configuration ──────────────────────────────────────────────────────

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/reports")
            // PDF de documentos fiscales
            .route("/invoices/{id}/pdf", web::get().to(get_invoice_pdf))
            .route("/credit-notes/{id}/pdf", web::get().to(get_credit_note_pdf))
            .route("/debit-notes/{id}/pdf", web::get().to(get_debit_note_pdf))
            // PDF de comprobantes de retenciones
            .route(
                "/withholdings/iva/{id}/pdf",
                web::get().to(get_iva_withholding_voucher_pdf),
            )
            .route(
                "/withholdings/islr/{id}/pdf",
                web::get().to(get_islr_arc_pdf),
            )
            // Exportación XML retenciones IVA
            .route(
                "/withholdings/iva/xml",
                web::get().to(get_iva_withholding_xml),
            )
            // Exportación TXT retenciones ISLR
            .route(
                "/withholdings/islr/txt",
                web::get().to(get_islr_withholding_txt),
            )
            // Exportación de libros
            .route("/books/purchases", web::get().to(get_purchase_book_export))
            .route("/books/sales", web::get().to(get_sales_book_export)),
    );
}
