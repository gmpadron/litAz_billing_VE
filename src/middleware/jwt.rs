//! Validación JWT compatible con litAz_auth_service (NestJS).
//!
//! El auth service emite tokens con el payload:
//! ```json
//! {
//!   "sub": "user-uuid",
//!   "email": "user@example.com",
//!   "roles": ["ADMIN", "ACCOUNTANT"],
//!   "permissions": ["invoices:create", "invoices:read"],
//!   "sessionId": "session-uuid",
//!   "deviceId": "fingerprint",
//!   "twoFactorVerified": true,
//!   "iss": "ecommerce-api",
//!   "aud": "ecommerce-frontend",
//!   "iat": 1234567890,
//!   "exp": 1234567890
//! }
//! ```

use jsonwebtoken::{DecodingKey, TokenData, Validation, decode};
use serde::{Deserialize, Serialize};

/// Claims extraídos del JWT emitido por litAz_auth_service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// User ID (UUID string)
    pub sub: String,
    /// Email del usuario
    #[serde(default)]
    pub email: String,
    /// Roles del usuario (ej: ["admin", "accountant", "viewer"])
    #[serde(default)]
    pub roles: Vec<String>,
    /// Permisos granulares (ej: ["invoices:create", "invoices:read"])
    #[serde(default)]
    pub permissions: Vec<String>,
    /// ID de la sesión (refresh token ID)
    #[serde(default, rename = "sessionId")]
    pub session_id: Option<String>,
    /// Device fingerprint
    #[serde(default, rename = "deviceId")]
    pub device_id: Option<String>,
    /// Si el usuario verificó 2FA en esta sesión
    #[serde(default, rename = "twoFactorVerified")]
    pub two_factor_verified: bool,
    /// Issued at (timestamp)
    #[serde(default)]
    pub iat: usize,
    /// Expiration (timestamp)
    #[serde(default)]
    pub exp: usize,
    /// Issuer
    #[serde(default)]
    pub iss: Option<String>,
    /// Audience
    #[serde(default)]
    pub aud: Option<String>,
}

/// Configuración para validación JWT.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: Option<String>,
    pub audience: Option<String>,
}

/// Decodifica y valida un token JWT.
pub fn validate_token(token: &str, config: &JwtConfig) -> Result<TokenData<JwtClaims>, AuthError> {
    let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());
    let mut validation = Validation::default();

    // Configurar issuer si está definido (si no se configura, no se valida)
    if let Some(ref issuer) = config.issuer {
        validation.set_issuer(&[issuer]);
    }

    // Configurar audience si está definida (si no se configura, no se valida)
    if let Some(ref audience) = config.audience {
        validation.set_audience(&[audience]);
    }

    decode::<JwtClaims>(token, &decoding_key, &validation).map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
        jsonwebtoken::errors::ErrorKind::InvalidToken => AuthError::InvalidToken,
        jsonwebtoken::errors::ErrorKind::InvalidIssuer => AuthError::InvalidIssuer,
        jsonwebtoken::errors::ErrorKind::InvalidAudience => AuthError::InvalidAudience,
        _ => AuthError::ValidationFailed,
    })
}

/// Extrae el token JWT del header Authorization (Bearer) o del cookie accessToken.
/// Orden de prioridad: header Authorization primero, luego cookie (igual que el API service NestJS).
pub fn extract_bearer_token(req: &actix_web::HttpRequest) -> Option<String> {
    // 1. Intentar desde el header Authorization: Bearer <token>
    if let Some(token) = req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .filter(|h| h.starts_with("Bearer "))
        .map(|h| h[7..].to_string())
    {
        return Some(token);
    }

    // 2. Fallback: leer desde el cookie accessToken (enviado por el frontend via credentials: 'include')
    req.cookie("accessToken").map(|c| c.value().to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Token no proporcionado")]
    MissingToken,

    #[error("Token inválido")]
    InvalidToken,

    #[error("Token expirado")]
    TokenExpired,

    #[error("Emisor del token inválido")]
    InvalidIssuer,

    #[error("Audiencia del token inválida")]
    InvalidAudience,

    #[error("Error de validación del token")]
    ValidationFailed,
}

impl From<AuthError> for crate::errors::AppError {
    fn from(err: AuthError) -> Self {
        crate::errors::AppError::Unauthorized(err.to_string())
    }
}
