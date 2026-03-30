# middleware/ — Autenticación JWT y Control de Acceso

Middleware de Actix Web que valida tokens JWT emitidos por `litAz_auth_service` e implementa control de acceso basado en roles (RBAC).

## Archivos

| Archivo        | Descripción                                                                          |
|----------------|--------------------------------------------------------------------------------------|
| `mod.rs`       | Re-exporta `JwtMiddleware`, `JwtConfig`, `AuthenticatedUser`, `require_admin`, `require_accountant`. |
| `jwt.rs`       | Lógica de validación JWT: `JwtClaims`, `JwtConfig`, `validate_token`, `AuthError`.  |
| `auth.rs`      | `JwtMiddleware` (Actix `Transform`/`Service`), `AuthenticatedUser` extractor, funciones `require_*`. |

## Flujo de autenticación

1. Cada request pasa por `JwtMiddleware`.
2. El middleware extrae el token del header `Authorization: Bearer <token>`.
3. Si no hay token → el request continúa pero sin claims en las extensions (los handlers que requieren auth fallan con 401).
4. Si hay token → se valida con `validate_token()` y los `JwtClaims` se insertan en `req.extensions`.
5. Los handlers que necesitan autenticación usan el extractor `AuthenticatedUser`, que lee los claims de las extensions.

## JwtClaims

Payload del JWT emitido por `litAz_auth_service`:

```rust
pub struct JwtClaims {
    pub sub: String,               // User ID (UUID)
    pub email: String,
    pub roles: Vec<String>,        // ej: ["ADMIN", "INFRA"]
    pub permissions: Vec<String>,  // ej: ["invoices:create", "invoices:read"]
    pub session_id: Option<String>,
    pub device_id: Option<String>,
    pub two_factor_verified: bool,
    pub iat: usize,
    pub exp: usize,
    pub iss: Option<String>,
    pub aud: Option<String>,
}
```

## AuthenticatedUser

Extractor de Actix Web para handlers que requieren autenticación. Lee los `JwtClaims` de las extensions del request y retorna `AppError::Unauthorized` si no están presentes.

```rust
// En cualquier handler que requiera auth:
async fn mi_handler(user: AuthenticatedUser, ...) -> Result<HttpResponse, AppError> {
    println!("User ID: {}", user.id);       // UUID del usuario
    println!("Email: {}", user.email);
    println!("Roles: {:?}", user.roles);    // Vec<String>
    // ...
}
```

## Funciones de control de acceso

### `require_admin(&user) -> Result<(), AppError>`

Permite el acceso solo a usuarios con rol `admin` o `ADMIN`. Retorna `AppError::Forbidden` si el usuario no tiene el rol.

```rust
async fn update_company(user: AuthenticatedUser, ...) -> Result<HttpResponse, AppError> {
    require_admin(&user)?;   // Solo admin puede modificar la empresa
    // ...
}
```

### `require_accountant(&user) -> Result<(), AppError>`

Permite el acceso a usuarios con rol `admin`, `ADMIN`, `accountant` o `ACCOUNTANT`. Retorna `AppError::Forbidden` si no cumple.

```rust
async fn create_invoice(user: AuthenticatedUser, ...) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;  // Admin o accountant pueden crear facturas
    // ...
}
```

Los usuarios sin ninguno de estos roles tienen acceso de solo lectura (cualquier GET sin restricción de rol pasa con cualquier JWT válido).

## Errores de autenticación

| Error              | HTTP | Descripción                                    |
|--------------------|------|------------------------------------------------|
| `MissingToken`     | 401  | No se proporcionó el header Authorization.     |
| `InvalidToken`     | 401  | El token no es un JWT válido.                  |
| `TokenExpired`     | 401  | El token expiró (`exp` en el pasado).          |
| `InvalidIssuer`    | 401  | El `iss` no coincide con `JWT_ISSUER`.         |
| `InvalidAudience`  | 401  | El `aud` no coincide con `JWT_AUDIENCE`.       |
| `ValidationFailed` | 401  | Error genérico de validación JWT.              |

## Modo desarrollo

En builds de debug, si no hay un JWT válido, los handlers pueden usar el header `X-User-Id` como fallback. Si tampoco está presente, se usa el UUID `11111111-1111-1111-1111-111111111111` para facilitar pruebas locales.

```bash
# Prueba sin token JWT (solo en cargo run / debug)
curl -H "X-User-Id: mi-uuid" http://localhost:8080/billingVE/v1/invoices
```

Este fallback **no existe** en builds de release.

## Configuración JWT

El middleware se inicializa en `main.rs` con:

```rust
let jwt_config = JwtConfig {
    secret: settings.jwt_secret.clone(),
    issuer: settings.jwt_issuer.clone(),   // None si JWT_ISSUER no está definido
    audience: settings.jwt_audience.clone(), // None si JWT_AUDIENCE no está definido
};

App::new()
    .wrap(JwtMiddleware::new(jwt_config.clone()))
    // ...
```

Si `JWT_ISSUER` y `JWT_AUDIENCE` son `None`, esos campos del token no se validan (flexible para entornos de desarrollo).
