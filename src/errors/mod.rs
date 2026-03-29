use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

/// Códigos de error estándar del sistema.
#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}

/// Error principal de la aplicación.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("No encontrado: {0}")]
    NotFound(String),

    #[error("Solicitud inválida: {0}")]
    BadRequest(String),

    #[error("No autorizado: {0}")]
    Unauthorized(String),

    #[error("Prohibido: {0}")]
    Forbidden(String),

    #[error("Error de validación: {0}")]
    Validation(String),

    #[error("Error interno: {0}")]
    Internal(String),

    #[error("Error de base de datos: {0}")]
    Database(#[from] sea_orm::DbErr),
}

impl AppError {
    fn error_code(&self) -> &str {
        match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Database(_) => "DATABASE_ERROR",
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        // Sanitizar: los errores de DB no deben exponer detalles internos al cliente.
        let message = match self {
            AppError::Database(db_err) => {
                log::error!("Database error: {}", db_err);
                "Error interno del servidor".to_string()
            }
            AppError::Internal(msg) => {
                log::error!("Internal error: {}", msg);
                "Error interno del servidor".to_string()
            }
            other => other.to_string(),
        };

        let body = serde_json::json!({
            "success": false,
            "error": {
                "code": self.error_code(),
                "message": message
            }
        });

        match self {
            AppError::NotFound(_) => HttpResponse::NotFound().json(body),
            AppError::BadRequest(_) | AppError::Validation(_) => {
                HttpResponse::BadRequest().json(body)
            }
            AppError::Unauthorized(_) => HttpResponse::Unauthorized().json(body),
            AppError::Forbidden(_) => HttpResponse::Forbidden().json(body),
            AppError::Internal(_) | AppError::Database(_) => {
                HttpResponse::InternalServerError().json(body)
            }
        }
    }
}
