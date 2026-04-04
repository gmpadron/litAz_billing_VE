//! Handlers HTTP para retenciones de IVA e ISLR.

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;

use crate::dto::ApiResponse;
use crate::dto::withholding_islr_dto::{CreateIslrWithholdingRequest, IslrWithholdingFilters};
use crate::dto::withholding_iva_dto::{CreateIvaWithholdingRequest, IvaWithholdingFilters};
use crate::errors::AppError;
use crate::middleware::{ActiveCompanyId, AuthenticatedUser, require_accountant};
use crate::services::{withholding_islr_service, withholding_iva_service};

// --- IVA Withholding Handlers ---

/// POST /billingVE/v1/withholdings/iva — requiere admin o accountant
async fn create_iva_withholding(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateIvaWithholdingRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let withholding = withholding_iva_service::create_iva_withholding(
        db.get_ref(),
        body.into_inner(),
        user.id,
        company.0,
    )
    .await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(withholding)))
}

/// GET /billingVE/v1/withholdings/iva — cualquier rol autenticado
async fn list_iva_withholdings(
    _user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    query: web::Query<IvaWithholdingFilters>,
) -> Result<HttpResponse, AppError> {
    let result =
        withholding_iva_service::list_iva_withholdings(db.get_ref(), query.into_inner(), company.0)
            .await?;
    Ok(HttpResponse::Ok().json(result))
}

/// GET /billingVE/v1/withholdings/iva/{id} — cualquier rol autenticado
async fn get_iva_withholding(
    _user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let withholding =
        withholding_iva_service::get_iva_withholding(db.get_ref(), path.into_inner(), company.0)
            .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(withholding)))
}

// --- ISLR Withholding Handlers ---

/// POST /billingVE/v1/withholdings/islr — requiere admin o accountant
async fn create_islr_withholding(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateIslrWithholdingRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let withholding = withholding_islr_service::create_islr_withholding(
        db.get_ref(),
        body.into_inner(),
        user.id,
        company.0,
    )
    .await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(withholding)))
}

/// GET /billingVE/v1/withholdings/islr — cualquier rol autenticado
async fn list_islr_withholdings(
    _user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    query: web::Query<IslrWithholdingFilters>,
) -> Result<HttpResponse, AppError> {
    let result = withholding_islr_service::list_islr_withholdings(
        db.get_ref(),
        query.into_inner(),
        company.0,
    )
    .await?;
    Ok(HttpResponse::Ok().json(result))
}

/// GET /billingVE/v1/withholdings/islr/{id} — cualquier rol autenticado
async fn get_islr_withholding(
    _user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let withholding = withholding_islr_service::get_islr_withholding(
        db.get_ref(),
        path.into_inner(),
        company.0,
    )
    .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(withholding)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/withholdings")
            .route("/iva", web::post().to(create_iva_withholding))
            .route("/iva", web::get().to(list_iva_withholdings))
            .route("/iva/{id}", web::get().to(get_iva_withholding))
            .route("/islr", web::post().to(create_islr_withholding))
            .route("/islr", web::get().to(list_islr_withholdings))
            .route("/islr/{id}", web::get().to(get_islr_withholding)),
    );
}
