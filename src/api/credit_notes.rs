//! Handlers HTTP para notas de crédito.

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;

use crate::dto::ApiResponse;
use crate::dto::credit_note_dto::{CreateCreditNoteRequest, CreditNoteFilters};
use crate::errors::AppError;
use crate::middleware::{
    ActiveCompanyId, AuthenticatedUser, require_accountant, require_billing_viewer,
};
use crate::services::credit_note_service;

/// POST /billingVE/v1/credit-notes — requiere admin o accountant
async fn create_credit_note(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateCreditNoteRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let note = credit_note_service::create_credit_note(
        db.get_ref(),
        body.into_inner(),
        user.id,
        company.0,
    )
    .await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(note)))
}

/// GET /billingVE/v1/credit-notes — admin/accountant/auditor/infra
async fn list_credit_notes(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    query: web::Query<CreditNoteFilters>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    let result =
        credit_note_service::list_credit_notes(db.get_ref(), query.into_inner(), company.0).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// GET /billingVE/v1/credit-notes/{id} — admin/accountant/auditor/infra
async fn get_credit_note(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    let note =
        credit_note_service::get_credit_note(db.get_ref(), path.into_inner(), company.0).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(note)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/credit-notes")
            .route("", web::post().to(create_credit_note))
            .route("", web::get().to(list_credit_notes))
            .route("/{id}", web::get().to(get_credit_note)),
    );
}
