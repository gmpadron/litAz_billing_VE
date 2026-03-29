//! Handlers HTTP para notas de crédito.

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;

use crate::api::helpers::get_active_company_id;
use crate::dto::ApiResponse;
use crate::dto::credit_note_dto::{CreateCreditNoteRequest, CreditNoteFilters};
use crate::errors::AppError;
use crate::middleware::{AuthenticatedUser, require_accountant};
use crate::services::credit_note_service;

/// POST /billingVE/v1/credit-notes — requiere admin o accountant
async fn create_credit_note(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateCreditNoteRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let company_id = get_active_company_id(db.get_ref()).await?;
    let note = credit_note_service::create_credit_note(
        db.get_ref(),
        body.into_inner(),
        user.id,
        company_id,
    )
    .await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(note)))
}

/// GET /billingVE/v1/credit-notes — cualquier rol autenticado
async fn list_credit_notes(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    query: web::Query<CreditNoteFilters>,
) -> Result<HttpResponse, AppError> {
    let result = credit_note_service::list_credit_notes(db.get_ref(), query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// GET /billingVE/v1/credit-notes/{id} — cualquier rol autenticado
async fn get_credit_note(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let note = credit_note_service::get_credit_note(db.get_ref(), path.into_inner()).await?;
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
