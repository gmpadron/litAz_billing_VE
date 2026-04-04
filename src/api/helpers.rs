//! Funciones compartidas para los handlers de la API.

use actix_web::HttpMessage;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::jwt::JwtClaims;

/// Extrae el user_id del JWT validado (claims en extensions).
///
/// En builds de producción requiere un JWT válido.
/// En builds de debug permite fallback con header `X-User-Id` para pruebas.
pub fn extract_user_id(req: &actix_web::HttpRequest) -> Result<Uuid, AppError> {
    // Intentar extraer del JWT (insertado por JwtMiddleware)
    if let Some(claims) = req.extensions().get::<JwtClaims>() {
        return claims.sub.parse::<Uuid>().map_err(|_| {
            AppError::Unauthorized("Token JWT contiene un user ID inválido".to_string())
        });
    }

    // Fallback solo en debug: leer header X-User-Id
    #[cfg(debug_assertions)]
    {
        let parsed = req
            .headers()
            .get("X-User-Id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<Uuid>().ok());

        match parsed {
            Some(id) => Ok(id),
            None => Ok("11111111-1111-1111-1111-111111111111"
                .parse()
                .expect("hardcoded UUID")),
        }
    }

    #[cfg(not(debug_assertions))]
    {
        Err(AppError::Unauthorized(
            "No autenticado. Token JWT requerido.".to_string(),
        ))
    }
}
