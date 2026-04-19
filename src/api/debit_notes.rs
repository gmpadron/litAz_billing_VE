//! Handlers HTTP para notas de débito.

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;

use crate::dto::ApiResponse;
use crate::dto::debit_note_dto::{CreateDebitNoteRequest, DebitNoteFilters};
use crate::errors::AppError;
use crate::middleware::{
    ActiveCompanyId, AuthenticatedUser, require_accountant, require_billing_viewer,
};
use crate::services::debit_note_service;

/// POST /billingVE/v1/debit-notes — requiere admin o accountant
async fn create_debit_note(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateDebitNoteRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let note = debit_note_service::create_debit_note(
        db.get_ref(),
        body.into_inner(),
        user.id,
        company.0,
    )
    .await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(note)))
}

/// GET /billingVE/v1/debit-notes — admin/accountant/auditor/infra
async fn list_debit_notes(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    query: web::Query<DebitNoteFilters>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    let result =
        debit_note_service::list_debit_notes(db.get_ref(), query.into_inner(), company.0).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// GET /billingVE/v1/debit-notes/{id} — admin/accountant/auditor/infra
async fn get_debit_note(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    let note =
        debit_note_service::get_debit_note(db.get_ref(), path.into_inner(), company.0).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(note)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/debit-notes")
            .route("", web::post().to(create_debit_note))
            .route("", web::get().to(list_debit_notes))
            .route("/{id}", web::get().to(get_debit_note)),
    );
}
