//! Middleware que valida el header `X-Company-ID` en cada request de billing.
//!
//! Lee el UUID de empresa del header, verifica que exista y esté activa en la BD,
//! e inserta `ActiveCompanyId` en las extensions para que los handlers lo consuman.
//!
//! Las rutas de gestión de empresa (`/company`, `/company/{id}`) y `/health` se omiten.

use std::future::{Ready, ready};
use std::rc::Rc;
use std::sync::Arc;

use actix_web::body::EitherBody;
use actix_web::dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use futures_util::future::LocalBoxFuture;
use sea_orm::{DatabaseConnection, EntityTrait};
use uuid::Uuid;

use crate::entities::company_profiles;
use crate::errors::AppError;

// ── Newtype ───────────────────────────────────────────────────────────────────

/// UUID de empresa validado, inyectado por `CompanyMiddleware`.
#[derive(Debug, Clone, Copy)]
pub struct ActiveCompanyId(pub Uuid);

/// Extractor para handlers: lee `ActiveCompanyId` de las extensions.
impl actix_web::FromRequest for ActiveCompanyId {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let result = req
            .extensions()
            .get::<ActiveCompanyId>()
            .copied()
            .ok_or_else(|| {
                AppError::BadRequest(
                    "Header X-Company-ID requerido. Seleccione una empresa.".to_string(),
                )
                .into()
            });
        ready(result)
    }
}

// ── Middleware factory ────────────────────────────────────────────────────────

pub struct CompanyMiddleware {
    db: Arc<DatabaseConnection>,
}

impl CompanyMiddleware {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CompanyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = CompanyMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CompanyMiddlewareService {
            service: Rc::new(service),
            db: self.db.clone(),
        }))
    }
}

// ── Middleware service ────────────────────────────────────────────────────────

pub struct CompanyMiddlewareService<S> {
    service: Rc<S>,
    db: Arc<DatabaseConnection>,
}

/// Rutas que no requieren X-Company-ID (gestión de empresas y health).
fn should_skip(path: &str) -> bool {
    let p = path.trim_end_matches('/');
    p == "/health"
        || p == "/billingVE/v1/health"
        || p == "/billingVE/v1/company"
        || p.starts_with("/billingVE/v1/company/")
}

impl<S, B> Service<ServiceRequest> for CompanyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if should_skip(req.path()) {
            let svc = self.service.clone();
            return Box::pin(async move {
                svc.call(req).await.map(|r| r.map_into_left_body())
            });
        }

        let svc = self.service.clone();
        let db = self.db.clone();

        Box::pin(async move {
            // Leer y parsear header X-Company-ID
            let company_uuid = match req
                .headers()
                .get("x-company-id")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| Uuid::parse_str(s).ok())
            {
                Some(id) => id,
                None => {
                    let err = AppError::BadRequest(
                        "Header X-Company-ID requerido. Seleccione una empresa desde el panel."
                            .to_string(),
                    );
                    let (http_req, _) = req.into_parts();
                    let resp = HttpResponse::from_error(err)
                        .map_into_right_body();
                    return Ok(ServiceResponse::new(http_req, resp));
                }
            };

            // Validar que la empresa exista y esté activa
            let found = company_profiles::Entity::find_by_id(company_uuid)
                .one(db.as_ref())
                .await;

            match found {
                Ok(Some(c)) if c.is_active => {
                    req.extensions_mut().insert(ActiveCompanyId(company_uuid));
                    svc.call(req).await.map(|r| r.map_into_left_body())
                }
                Ok(_) => {
                    let err = AppError::BadRequest(format!(
                        "Empresa con ID {} no encontrada o inactiva.",
                        company_uuid
                    ));
                    let (http_req, _) = req.into_parts();
                    let resp = HttpResponse::from_error(err)
                        .map_into_right_body();
                    Ok(ServiceResponse::new(http_req, resp))
                }
                Err(e) => {
                    let err = AppError::Internal(format!("Error validando empresa: {}", e));
                    let (http_req, _) = req.into_parts();
                    let resp = HttpResponse::from_error(err)
                        .map_into_right_body();
                    Ok(ServiceResponse::new(http_req, resp))
                }
            }
        })
    }
}
