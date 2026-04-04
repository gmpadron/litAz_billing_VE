//! Handlers HTTP para Libros de Compras y Ventas.

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;

use crate::dto::ApiResponse;
use crate::dto::book_dto::{
    BookFilters, CreatePurchaseBookEntryRequest, CreateSalesBookEntryRequest,
};
use crate::errors::AppError;
use crate::middleware::{ActiveCompanyId, AuthenticatedUser, require_accountant};
use crate::services::book_service;

/// GET /billingVE/v1/books/purchases?period=2026-03 — cualquier rol autenticado
async fn get_purchase_book(
    _user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    query: web::Query<BookFilters>,
) -> Result<HttpResponse, AppError> {
    let result =
        book_service::get_purchase_book(db.get_ref(), query.into_inner(), company.0).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
}

/// POST /billingVE/v1/books/purchases — requiere admin o accountant
async fn create_purchase_book_entry(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreatePurchaseBookEntryRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let entry = book_service::create_purchase_book_entry(
        db.get_ref(),
        body.into_inner(),
        user.id,
        company.0,
    )
    .await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(entry)))
}

/// GET /billingVE/v1/books/sales?period=2026-03 — cualquier rol autenticado
async fn get_sales_book(
    _user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    query: web::Query<BookFilters>,
) -> Result<HttpResponse, AppError> {
    let result = book_service::get_sales_book(db.get_ref(), query.into_inner(), company.0).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
}

/// POST /billingVE/v1/books/sales — requiere admin o accountant
async fn create_sales_book_entry(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateSalesBookEntryRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let entry = book_service::create_sales_book_entry(
        db.get_ref(),
        body.into_inner(),
        user.id,
        company.0,
    )
    .await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(entry)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/books")
            .route("/purchases", web::get().to(get_purchase_book))
            .route("/purchases", web::post().to(create_purchase_book_entry))
            .route("/sales", web::get().to(get_sales_book))
            .route("/sales", web::post().to(create_sales_book_entry)),
    );
}
