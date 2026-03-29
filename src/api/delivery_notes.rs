//! Handlers HTTP para guías de despacho / órdenes de entrega.

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;

use crate::api::helpers::get_active_company_id;
use crate::dto::ApiResponse;
use crate::dto::delivery_note_dto::{CreateDeliveryNoteRequest, DeliveryNoteFilters};
use crate::errors::AppError;
use crate::middleware::{AuthenticatedUser, require_accountant};
use crate::services::delivery_note_service;

/// POST /billingVE/v1/delivery-notes — requiere admin o accountant
async fn create_delivery_note(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateDeliveryNoteRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let company_id = get_active_company_id(db.get_ref()).await?;
    let note = delivery_note_service::create_delivery_note(
        db.get_ref(),
        body.into_inner(),
        user.id,
        company_id,
    )
    .await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(note)))
}

/// GET /billingVE/v1/delivery-notes — cualquier rol autenticado
async fn list_delivery_notes(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    query: web::Query<DeliveryNoteFilters>,
) -> Result<HttpResponse, AppError> {
    let result =
        delivery_note_service::list_delivery_notes(db.get_ref(), query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// GET /billingVE/v1/delivery-notes/{id} — cualquier rol autenticado
async fn get_delivery_note(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let note = delivery_note_service::get_delivery_note(db.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(note)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/delivery-notes")
            .route("", web::post().to(create_delivery_note))
            .route("", web::get().to(list_delivery_notes))
            .route("/{id}", web::get().to(get_delivery_note)),
    );
}
