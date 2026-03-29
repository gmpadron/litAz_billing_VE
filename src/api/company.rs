//! Handlers HTTP para el perfil fiscal de la empresa emisora.

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;

use crate::dto::ApiResponse;
use crate::dto::company_dto::UpdateCompanyRequest;
use crate::errors::AppError;
use crate::middleware::{AuthenticatedUser, require_admin};
use crate::services::company_service;

/// GET /billingVE/v1/company — cualquier rol autenticado
async fn get_company(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    let company = company_service::get_company(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(company)))
}

/// PUT /billingVE/v1/company — solo admin
async fn update_company(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<UpdateCompanyRequest>,
) -> Result<HttpResponse, AppError> {
    require_admin(&user)?;
    let company = company_service::update_company(db.get_ref(), body.into_inner(), user.id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(company)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/company")
            .route("", web::get().to(get_company))
            .route("", web::put().to(update_company)),
    );
}
