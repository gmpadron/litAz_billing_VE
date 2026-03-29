//! Handlers HTTP para notas de débito.

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;

use crate::api::helpers::get_active_company_id;
use crate::dto::ApiResponse;
use crate::dto::debit_note_dto::{CreateDebitNoteRequest, DebitNoteFilters};
use crate::errors::AppError;
use crate::middleware::{AuthenticatedUser, require_accountant};
use crate::services::debit_note_service;

/// POST /billingVE/v1/debit-notes — requiere admin o accountant
async fn create_debit_note(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateDebitNoteRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let company_id = get_active_company_id(db.get_ref()).await?;
    let note = debit_note_service::create_debit_note(
        db.get_ref(),
        body.into_inner(),
        user.id,
        company_id,
    )
    .await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(note)))
}

/// GET /billingVE/v1/debit-notes — cualquier rol autenticado
async fn list_debit_notes(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    query: web::Query<DebitNoteFilters>,
) -> Result<HttpResponse, AppError> {
    let result = debit_note_service::list_debit_notes(db.get_ref(), query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// GET /billingVE/v1/debit-notes/{id} — cualquier rol autenticado
async fn get_debit_note(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let note = debit_note_service::get_debit_note(db.get_ref(), path.into_inner()).await?;
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
