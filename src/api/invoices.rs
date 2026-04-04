//! Handlers HTTP para facturas.
//!
//! Las facturas son INMUTABLES una vez emitidas (PA SNAT/2011/0071).
//! Para corregir o revertir una factura emitida se debe emitir una Nota de Crédito.

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;

use crate::api::helpers::get_active_company_id;
use crate::dto::ApiResponse;
use crate::dto::invoice_dto::{CreateInvoiceRequest, InvoiceFilters};
use crate::errors::AppError;
use crate::middleware::{AuthenticatedUser, require_accountant};
use crate::services::invoice_service;

/// POST /billingVE/v1/invoices — requiere admin o accountant
async fn create_invoice(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateInvoiceRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let company_id = get_active_company_id(db.get_ref()).await?;
    let invoice =
        invoice_service::create_invoice(db.get_ref(), body.into_inner(), user.id, company_id)
            .await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(invoice)))
}

/// GET /billingVE/v1/invoices — cualquier rol autenticado
async fn list_invoices(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    query: web::Query<InvoiceFilters>,
) -> Result<HttpResponse, AppError> {
    let result = invoice_service::list_invoices(db.get_ref(), query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// GET /billingVE/v1/invoices/{id} — cualquier rol autenticado
async fn get_invoice(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let invoice = invoice_service::get_invoice(db.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(invoice)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/invoices")
            .route("", web::post().to(create_invoice))
            .route("", web::get().to(list_invoices))
            .route("/{id}", web::get().to(get_invoice)),
    );
}
