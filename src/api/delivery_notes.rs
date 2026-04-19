//! Handlers HTTP para guías de despacho / órdenes de entrega.

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;

use crate::dto::ApiResponse;
use crate::dto::delivery_note_dto::{CreateDeliveryNoteRequest, DeliveryNoteFilters};
use crate::errors::AppError;
use crate::middleware::{
    ActiveCompanyId, AuthenticatedUser, require_accountant, require_billing_viewer,
};
use crate::services::delivery_note_service;

/// POST /billingVE/v1/delivery-notes — requiere admin o accountant
async fn create_delivery_note(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateDeliveryNoteRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let note = delivery_note_service::create_delivery_note(
        db.get_ref(),
        body.into_inner(),
        user.id,
        company.0,
    )
    .await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(note)))
}

/// GET /billingVE/v1/delivery-notes — admin/accountant/auditor/infra
async fn list_delivery_notes(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    query: web::Query<DeliveryNoteFilters>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    let result =
        delivery_note_service::list_delivery_notes(db.get_ref(), query.into_inner(), company.0)
            .await?;
    Ok(HttpResponse::Ok().json(result))
}

/// GET /billingVE/v1/delivery-notes/{id} — admin/accountant/auditor/infra
async fn get_delivery_note(
    user: AuthenticatedUser,
    company: ActiveCompanyId,
    db: web::Data<DatabaseConnection>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    require_billing_viewer(&user)?;
    let note =
        delivery_note_service::get_delivery_note(db.get_ref(), path.into_inner(), company.0)
            .await?;
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
