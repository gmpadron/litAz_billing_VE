# api/ — Handlers HTTP

Capa de presentación de la aplicación. Cada archivo contiene los handlers Actix Web de un recurso y su función `configure()` que registra las rutas.

Los handlers son deliberadamente delgados: deserializan el request, invocan el service correspondiente y serializan la respuesta. Toda la lógica de negocio vive en `domain/` y `services/`.

## Archivos

| Archivo             | Scope de rutas          | Descripción                                          |
|---------------------|-------------------------|------------------------------------------------------|
| `mod.rs`            | `/billingVE/v1`         | Registra todos los sub-módulos.                      |
| `invoices.rs`       | `/invoices`             | CRUD de facturas + anulación.                        |
| `credit_notes.rs`   | `/credit-notes`         | CRUD de notas de crédito.                            |
| `debit_notes.rs`    | `/debit-notes`          | CRUD de notas de débito.                             |
| `delivery_notes.rs` | `/delivery-notes`       | CRUD de guías de despacho.                           |
| `clients.rs`        | `/clients`              | CRUD de clientes.                                    |
| `company.rs`        | `/company`              | Perfil fiscal del emisor.                            |
| `books.rs`          | `/books`                | Libros de compras y ventas.                          |
| `withholdings.rs`   | `/withholdings`         | Retenciones IVA e ISLR.                              |
| `reports.rs`        | `/reports`              | Exportaciones PDF, XML y TXT.                        |
| `helpers.rs`        | —                       | `get_active_company_id`, `extract_user_id`.          |

## Rutas

### `/invoices`

| Método | Ruta          | Rol        | Descripción                                            |
|--------|---------------|------------|--------------------------------------------------------|
| `POST` | `/invoices`   | accountant | Crea una factura. Asigna número de control automático. |
| `GET`  | `/invoices`   | viewer     | Lista facturas con filtros y paginación.               |
| `GET`  | `/{id}`       | viewer     | Retorna detalle completo con items.                    |
| `POST` | `/{id}/void`  | accountant | Anula la factura. Requiere motivo en el body.          |

### `/credit-notes`

| Método | Ruta   | Rol        | Descripción                                              |
|--------|--------|------------|----------------------------------------------------------|
| `POST` | `/`    | accountant | Crea nota de crédito vinculada a una factura existente.  |
| `GET`  | `/`    | viewer     | Lista notas de crédito.                                  |
| `GET`  | `/{id}`| viewer     | Detalle de nota de crédito.                              |

### `/debit-notes`

| Método | Ruta   | Rol        | Descripción                                             |
|--------|--------|------------|---------------------------------------------------------|
| `POST` | `/`    | accountant | Crea nota de débito vinculada a una factura existente.  |
| `GET`  | `/`    | viewer     | Lista notas de débito.                                  |
| `GET`  | `/{id}`| viewer     | Detalle de nota de débito.                              |

### `/delivery-notes`

| Método | Ruta   | Rol        | Descripción               |
|--------|--------|------------|---------------------------|
| `POST` | `/`    | accountant | Crea guía de despacho.    |
| `GET`  | `/`    | viewer     | Lista guías de despacho.  |
| `GET`  | `/{id}`| viewer     | Detalle de guía.          |

### `/clients`

| Método | Ruta   | Rol        | Descripción                                      |
|--------|--------|------------|--------------------------------------------------|
| `POST` | `/`    | accountant | Crea cliente. Valida formato de RIF si se provee.|
| `GET`  | `/`    | viewer     | Lista clientes.                                  |
| `GET`  | `/{id}`| viewer     | Detalle de cliente.                              |
| `PUT`  | `/{id}`| accountant | Actualiza datos del cliente.                     |

### `/company`

| Método | Ruta | Rol    | Descripción                                       |
|--------|------|--------|---------------------------------------------------|
| `GET`  | `/`  | viewer | Retorna el perfil fiscal activo del emisor.       |
| `PUT`  | `/`  | admin  | Actualiza razón social, RIF, domicilio, etc.      |

### `/books`

| Método | Ruta         | Rol        | Descripción                                        |
|--------|--------------|------------|----------------------------------------------------|
| `GET`  | `/purchases` | viewer     | Libro de compras del período `?period=YYYY-MM`.    |
| `POST` | `/purchases` | accountant | Registra entrada manual en libro de compras.       |
| `GET`  | `/sales`     | viewer     | Libro de ventas del período `?period=YYYY-MM`.     |
| `POST` | `/sales`     | accountant | Registra entrada manual en libro de ventas.        |

### `/withholdings`

| Método | Ruta        | Rol        | Descripción                       |
|--------|-------------|------------|-----------------------------------|
| `POST` | `/iva`      | accountant | Crea retención IVA.               |
| `GET`  | `/iva`      | viewer     | Lista retenciones IVA.            |
| `GET`  | `/iva/{id}` | viewer     | Detalle de retención IVA.         |
| `POST` | `/islr`     | accountant | Crea retención ISLR.              |
| `GET`  | `/islr`     | viewer     | Lista retenciones ISLR.           |
| `GET`  | `/islr/{id}`| viewer     | Detalle de retención ISLR.        |

### `/reports`

| Método | Ruta                                     | Descripción                                    |
|--------|------------------------------------------|------------------------------------------------|
| `GET`  | `/invoices/{id}/pdf`                     | Descarga factura en PDF.                       |
| `GET`  | `/credit-notes/{id}/pdf`                 | Descarga nota de crédito en PDF.               |
| `GET`  | `/debit-notes/{id}/pdf`                  | Descarga nota de débito en PDF.                |
| `GET`  | `/withholdings/iva/{id}/pdf`             | Comprobante de retención IVA en PDF.           |
| `GET`  | `/withholdings/islr/{id}/pdf`            | Comprobante ARC ISLR en PDF.                   |
| `GET`  | `/withholdings/iva/xml?period=202603-01` | XML quincenal retenciones IVA (formato SENIAT).|
| `GET`  | `/withholdings/islr/txt?period=2026-03`  | TXT mensual retenciones ISLR (formato SENIAT). |

## helpers.rs

### `get_active_company_id(db) -> Result<Uuid, AppError>`

Busca el primer `company_profiles` con `is_active = true`. Si no existe, retorna `AppError::BadRequest` con el mensaje `"No hay perfil de empresa activo"`.

Todos los handlers de creación de documentos llaman a esta función para asociar el documento al perfil fiscal vigente.

### `extract_user_id(req) -> Result<Uuid, AppError>`

Extrae el `sub` (user UUID) del JWT en las extensions del request. En builds de debug, acepta el header `X-User-Id` como fallback.

## Patrón de un handler

```rust
async fn create_invoice(
    user: AuthenticatedUser,          // 1. Extrae JWT claims — 401 si no está autenticado
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateInvoiceRequest>,
) -> Result<HttpResponse, AppError> {
    require_accountant(&user)?;       // 2. Verifica rol — 403 si no es admin/accountant
    let company_id = get_active_company_id(db.get_ref()).await?;  // 3. Obtiene empresa activa
    let invoice = invoice_service::create_invoice(              // 4. Delega al service
        db.get_ref(), body.into_inner(), user.id, company_id
    ).await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(invoice))) // 5. Serializa respuesta
}
```
