//! Handlers HTTP para clientes.

use actix_web::{HttpResponse, web};
use sea_orm::DatabaseConnection;

use crate::dto::ApiResponse;
use crate::dto::client_dto::{ClientFilters, CreateClientRequest, UpdateClientRequest};
use crate::errors::AppError;
use crate::middleware::{AuthenticatedUser, require_accountant};
use crate::services::client_service;

/// POST /billingVE/v1/clients — requiere admin o accountant
async fn create_client(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateClientRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let client = client_service::create_client(db.get_ref(), body.into_inner(), user.id).await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(client)))
}

/// GET /billingVE/v1/clients — cualquier rol autenticado
async fn list_clients(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    query: web::Query<ClientFilters>,
) -> Result<HttpResponse, AppError> {
    let result = client_service::list_clients(db.get_ref(), query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// GET /billingVE/v1/clients/{id} — cualquier rol autenticado
async fn get_client(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let client = client_service::get_client(db.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(client)))
}

/// PUT /billingVE/v1/clients/{id} — requiere admin o accountant
async fn update_client(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<uuid::Uuid>,
    body: web::Json<UpdateClientRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;
    let client =
        client_service::update_client(db.get_ref(), path.into_inner(), body.into_inner(), user.id)
            .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(client)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/clients")
            .route("", web::post().to(create_client))
            .route("", web::get().to(list_clients))
            .route("/{id}", web::get().to(get_client))
            .route("/{id}", web::put().to(update_client)),
    );
}
