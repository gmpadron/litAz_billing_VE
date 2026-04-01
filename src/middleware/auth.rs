//! Middleware de autenticación JWT para Actix Web.
//!
//! Valida el token JWT en cada request (excepto rutas públicas),
//! extrae los claims y los coloca en `req.extensions()` para que
//! los handlers los consuman via el extractor `AuthenticatedUser`.

use std::future::{Ready, ready};
use std::rc::Rc;

use actix_web::body::EitherBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, HttpResponse};
use log::warn;

use super::jwt::{JwtClaims, JwtConfig, extract_bearer_token, validate_token};

/// Middleware factory que valida JWT en cada request.
///
/// Se salta la validación si la ruta es `/health` o si el request
/// tiene la extensión `SkipAuth` (para rutas públicas futuras).
pub struct JwtMiddleware {
    config: JwtConfig,
}

impl JwtMiddleware {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = JwtMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService {
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
    config: JwtConfig,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();

        Box::pin(async move {
            // Rutas públicas que no requieren autenticación
            let path = req.path();
            if path == "/health" || path == "/billingVE/v1/health" {
                let res = service.call(req).await?.map_into_left_body();
                return Ok(res);
            }

            // En debug, permitir bypass con header X-User-Id para pruebas locales
            #[cfg(debug_assertions)]
            {
                if req.headers().get("X-User-Id").is_some()
                    && req.headers().get("Authorization").is_none()
                {
                    // Modo desarrollo: crear claims sintéticos desde el header
                    if let Some(user_id_str) = req
                        .headers()
                        .get("X-User-Id")
                        .and_then(|v| v.to_str().ok())
                    {
                        let dev_claims = JwtClaims {
                            sub: user_id_str.to_string(),
                            email: "dev@localhost".to_string(),
                            roles: vec!["admin".to_string()],
                            permissions: vec!["*:*".to_string()],
                            session_id: None,
                            device_id: None,
                            two_factor_verified: false,
                            iat: 0,
                            exp: 0,
                            iss: None,
                            aud: None,
                        };
                        req.extensions_mut().insert(dev_claims);
                        let res = service.call(req).await?.map_into_left_body();
                        return Ok(res);
                    }
                }
            }

            // Extraer y validar token
            let token = match extract_bearer_token(req.request()) {
                Some(t) => t,
                None => {
                    warn!("Request sin token JWT: {} {}", req.method(), req.path());
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "success": false,
                        "error": {
                            "code": "UNAUTHORIZED",
                            "message": "Token de autenticación requerido"
                        }
                    }));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            let claims = match validate_token(&token, &config) {
                Ok(data) => data.claims,
                Err(e) => {
                    warn!("JWT inválido en {} {}: {}", req.method(), req.path(), e);
                    let message = match e {
                        super::jwt::AuthError::TokenExpired => "Token expirado",
                        _ => "Token inválido",
                    };
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "success": false,
                        "error": {
                            "code": "UNAUTHORIZED",
                            "message": message
                        }
                    }));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            // Insertar claims en extensions del request
            req.extensions_mut().insert(claims);

            // Continuar con el handler
            let res = service.call(req).await?.map_into_left_body();
            Ok(res)
        })
    }
}

// =============================================================================
// Extractor AuthenticatedUser — se inyecta directamente en handlers
// =============================================================================

/// Usuario autenticado extraído del JWT validado.
///
/// Se usa como parámetro de handler en lugar de `extract_user_id()`:
/// ```rust,ignore
/// async fn create_invoice(
///     user: AuthenticatedUser,
///     db: web::Data<DatabaseConnection>,
///     body: web::Json<CreateInvoiceRequest>,
/// ) -> Result<HttpResponse, AppError> {
///     let user_id = user.id;
///     let roles = &user.roles;
///     // ...
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: uuid::Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub session_id: Option<String>,
    pub two_factor_verified: bool,
}

impl AuthenticatedUser {
    /// Verifica si el usuario tiene un rol específico.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Verifica si el usuario tiene un permiso específico.
    /// Soporta wildcards: `*:*` permite todo, `invoices:*` permite todo en invoices.
    pub fn has_permission(&self, permission: &str) -> bool {
        if self.permissions.iter().any(|p| p == "*:*") {
            return true;
        }
        if self.permissions.iter().any(|p| p == permission) {
            return true;
        }
        // Wildcard por recurso: "invoices:*" permite "invoices:create"
        if let Some(resource) = permission.split(':').next() {
            let wildcard = format!("{}:*", resource);
            if self.permissions.iter().any(|p| p == &wildcard) {
                return true;
            }
        }
        false
    }

    /// Verifica si el usuario tiene al menos uno de los roles indicados.
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|r| self.has_role(r))
    }
}

impl actix_web::FromRequest for AuthenticatedUser {
    type Error = crate::errors::AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let result = req
            .extensions()
            .get::<JwtClaims>()
            .cloned()
            .ok_or_else(|| {
                crate::errors::AppError::Unauthorized(
                    "No autenticado. Token JWT requerido.".to_string(),
                )
            })
            .and_then(|claims| {
                let id = claims.sub.parse::<uuid::Uuid>().map_err(|_| {
                    crate::errors::AppError::Unauthorized(
                        "Token JWT contiene un user ID inválido".to_string(),
                    )
                })?;

                Ok(AuthenticatedUser {
                    id,
                    email: claims.email,
                    roles: claims.roles,
                    permissions: claims.permissions,
                    session_id: claims.session_id,
                    two_factor_verified: claims.two_factor_verified,
                })
            });

        ready(result)
    }
}

// =============================================================================
// Guards de autorización por rol
// =============================================================================

/// Guard que verifica que el usuario tenga rol `ADMIN`, `ACCOUNTANT` o `INFRA`.
/// Se usa para endpoints de escritura fiscal.
pub fn require_accountant(user: &AuthenticatedUser) -> Result<(), crate::errors::AppError> {
    if user.has_any_role(&["ADMIN", "ACCOUNTANT", "INFRA"]) {
        Ok(())
    } else {
        Err(crate::errors::AppError::Forbidden(
            "Acceso denegado. Se requiere rol 'ADMIN' o 'ACCOUNTANT'.".to_string(),
        ))
    }
}

/// Guard que verifica que el usuario tenga rol `ADMIN` o `INFRA`.
/// Se usa para endpoints de configuración.
pub fn require_admin(user: &AuthenticatedUser) -> Result<(), crate::errors::AppError> {
    if user.has_any_role(&["ADMIN", "INFRA"]) {
        Ok(())
    } else {
        Err(crate::errors::AppError::Forbidden(
            "Acceso denegado. Se requiere rol 'ADMIN'.".to_string(),
        ))
    }
}
