//! Handlers HTTP para gestión de perfiles fiscales de empresa (multi-empresa).

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::dto::ApiResponse;
use crate::dto::company_dto::{CreateCompanyRequest, UpdateCompanyRequest};
use crate::errors::AppError;
use crate::middleware::{AuthenticatedUser, require_admin, require_billing_viewer};
use crate::services::company_service;

/// GET /billingVE/v1/company — admin/accountant/auditor/infra
async fn list_companies(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    let companies = company_service::list_companies(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(companies)))
}

/// GET /billingVE/v1/company/{id} — admin/accountant/auditor/infra
async fn get_company(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    let company = company_service::get_company_by_id(db.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(company)))
}

/// POST /billingVE/v1/company — crea una nueva empresa (solo admin)
async fn create_company(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateCompanyRequest>,
) -> Result<HttpResponse, AppError> {
    require_admin(&user)?;
    let company =
        company_service::create_company(db.get_ref(), body.into_inner(), user.id).await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(company)))
}

/// PUT /billingVE/v1/company/{id} — actualiza una empresa (solo admin)
async fn update_company(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateCompanyRequest>,
) -> Result<HttpResponse, AppError> {
    require_admin(&user)?;
    let company =
        company_service::update_company(db.get_ref(), path.into_inner(), body.into_inner(), user.id)
            .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(company)))
}

/// DELETE /billingVE/v1/company/{id} — desactiva una empresa (solo admin)
async fn deactivate_company(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_admin(&user)?;
    company_service::deactivate_company(db.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success("Empresa desactivada.")))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/company")
            .route("", web::get().to(list_companies))
            .route("", web::post().to(create_company))
            .route("/{id}", web::get().to(get_company))
            .route("/{id}", web::put().to(update_company))
            .route("/{id}", web::delete().to(deactivate_company)),
    );
}
