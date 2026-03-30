# LitAz Billing Core

Microservicio de facturación fiscal para Venezuela, conforme a las normativas del SENIAT. Gestiona facturas, notas de crédito/débito, guías de despacho, retenciones de IVA e ISLR, y libros fiscales. La autenticación es delegada a `litAz_auth_service`.

**Stack:** Rust (Edition 2024) · Actix Web 4 · SeaORM · PostgreSQL 16+ · JWT (HS256)

---

## Iniciar en desarrollo

### 1. Variables de entorno

Crea un archivo `.env` en la raíz basándote en `.env.test`:

```env
# Base de datos
DATABASE_URL=postgres://user:pass@localhost:5432/billing_core

# JWT — debe coincidir exactamente con litAz_auth_service
JWT_SECRET=tu_secreto_compartido_de_al_menos_32_chars
JWT_ISSUER=litaz-auth-service    # Opcional — si se define, valida el campo iss del JWT
JWT_AUDIENCE=litaz-ecosystem     # Opcional — si se define, valida el campo aud del JWT

# Servidor
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info

# CORS (orígenes separados por coma, vacío permite cualquiera en debug)
CORS_ORIGINS=http://localhost:4200,http://localhost:3000

# ── Seed inicial (opcional, solo para primera ejecución) ──
SEED_COMPANY_RAZON_SOCIAL=Mi Empresa C.A.
SEED_COMPANY_NOMBRE_COMERCIAL=Mi Empresa
SEED_COMPANY_RIF=J-12345678-9
SEED_COMPANY_DOMICILIO_FISCAL=Av. Principal, Caracas, Venezuela
SEED_COMPANY_TELEFONO=+58-212-5551234
SEED_COMPANY_EMAIL=facturacion@miempresa.com
SEED_COMPANY_ES_CONTRIBUYENTE_ESPECIAL=false
SEED_COMPANY_NRO_CONTRIBUYENTE_ESPECIAL=

SEED_CONTROL_PREFIX=00
SEED_CONTROL_RANGE_FROM=1
SEED_CONTROL_RANGE_TO=99999999
SEED_CONTROL_IMPRENTA=Imprenta Autorizada SENIAT S.A.
```

### 2. Base de datos

```bash
# Crear la base de datos (las migraciones corren automáticamente al iniciar)
createdb billing_core
```

Las 17 migraciones se ejecutan automáticamente con SeaORM en cada arranque del servidor.

### 3. Levantar el servicio

```bash
# Instalar Rust si no está disponible
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Compilar y ejecutar
cargo run
```

El servicio estará disponible en `http://localhost:8080/billingVE/v1`.
Health check en `http://localhost:8080/health`.

### 4. Modo desarrollo sin JWT

En builds de debug (`cargo run`), se puede omitir el token JWT y enviar el header `X-User-Id` directamente para pruebas locales:

```bash
curl -H "X-User-Id: 11111111-1111-1111-1111-111111111111" \
     http://localhost:8080/billingVE/v1/invoices
```

En builds de release (`cargo build --release`) este fallback está desactivado: se requiere un JWT válido en todas las rutas.

---

## Integración con litAz_auth_service

Este servicio no gestiona usuarios ni sesiones. Requiere que el cliente envíe el `accessToken` emitido por `litAz_auth_service` como Bearer token.

### Autenticar requests

```javascript
// El accessToken se obtiene del login en litAz_auth_service
const { accessToken } = loginResponse.data;

// Usarlo en todas las peticiones al servicio de billing
fetch('http://billing-service/billingVE/v1/invoices', {
  headers: {
    'Authorization': `Bearer ${accessToken}`,
    'Content-Type': 'application/json',
  },
});
```

### Payload JWT esperado

El `litAz_auth_service` emite tokens con este payload, que el servicio de billing lee para determinar roles y permisos:

```json
{
  "sub": "user-uuid",
  "email": "user@example.com",
  "roles": ["ADMIN"],
  "permissions": ["invoices:create", "invoices:read"],
  "sessionId": "session-uuid",
  "deviceId": "fingerprint",
  "twoFactorVerified": true,
  "iss": "litaz-auth-service",
  "aud": "litaz-ecosystem",
  "iat": 1234567890,
  "exp": 1234567890
}
```

### Sistema de roles

| Rol         | Acceso                                                      |
|-------------|-------------------------------------------------------------|
| `ADMIN`     | Acceso total. Puede modificar el perfil fiscal de empresa.  |
| `INFRA`     | Superusuario. Todos los permisos (wildcard `*:*`).          |
| `ADMIN` (cliente) | Crear/anular documentos, gestionar clientes y libros. |
| Cualquier autenticado | Solo lectura en todos los endpoints.            |

En los handlers, el acceso se verifica mediante las funciones `require_admin(&user)` y `require_accountant(&user)`. Un usuario sin estos roles solo puede hacer GET.

### Crear una factura desde el frontend

```javascript
const invoice = await fetch('http://billing-service/billingVE/v1/invoices', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${accessToken}`,
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    client_id: "uuid-del-cliente",
    items: [
      {
        description: "Servicio de consultoría",
        quantity: "1",
        unit_price: "1000.00",
        tax_rate: "16"
      }
    ],
    currency: "USD",
    exchange_rate: "36.50",
    notes: "Factura por servicios prestados"
  }),
});

const { success, data } = await invoice.json();
// data.invoice_number: "00-00000001"
// data.control_number: "00-00000001"
```

### Descargar un PDF de factura

```javascript
const response = await fetch(
  `http://billing-service/billingVE/v1/reports/invoices/${invoiceId}/pdf`,
  { headers: { 'Authorization': `Bearer ${accessToken}` } }
);
const blob = await response.blob();
const url = URL.createObjectURL(blob);
window.open(url); // Abre el PDF en nueva pestaña
```

---

## Arquitectura

```
src/
├── main.rs              # Entry point — servidor, migraciones, seed, CORS, JWT middleware
├── config/              # Settings (variables de entorno), conexión a BD
├── api/                 # Handlers HTTP (capa delgada, sin lógica de negocio)
│   ├── invoices.rs      # POST/GET /invoices, POST /{id}/void
│   ├── credit_notes.rs  # POST/GET /credit-notes
│   ├── debit_notes.rs   # POST/GET /debit-notes
│   ├── delivery_notes.rs # POST/GET /delivery-notes
│   ├── clients.rs       # CRUD /clients
│   ├── company.rs       # GET/PUT /company
│   ├── books.rs         # GET/POST /books/purchases y /books/sales
│   ├── withholdings.rs  # POST/GET /withholdings/iva y /withholdings/islr
│   ├── reports.rs       # PDF, XML, TXT para SENIAT
│   └── helpers.rs       # get_active_company_id, extract_user_id
├── domain/              # Lógica de negocio pura (sin I/O, sin frameworks)
│   ├── invoice/         # Builder de facturas, estados, validación fiscal
│   ├── numbering/       # RIF (validación dígito verificador), números de control, secuencias
│   ├── tax/             # Cálculos IVA e ISLR según decreto SENIAT
│   ├── withholding/     # Cálculos de retenciones IVA e ISLR
│   ├── books/           # Lógica libros de compras/ventas, resumen diario
│   └── currency/        # Tasa de cambio BCV (consulta API externa)
├── services/            # Orquestación: domain + BD + exportaciones
│   ├── invoice_service.rs
│   ├── credit_note_service.rs
│   ├── debit_note_service.rs
│   ├── delivery_note_service.rs
│   ├── client_service.rs
│   ├── company_service.rs
│   ├── book_service.rs
│   ├── withholding_iva_service.rs
│   ├── withholding_islr_service.rs
│   ├── pdf_service.rs   # Generación de PDFs con genpdf
│   ├── xml_service.rs   # XML retenciones IVA (formato SENIAT)
│   ├── txt_service.rs   # TXT retenciones ISLR (formato SENIAT)
│   ├── export_service.rs # Coordina exportaciones
│   └── seeder.rs        # Seed inicial de empresa y numeración
├── dto/                 # Request/Response structs (serde)
├── entities/            # Modelos SeaORM (15 tablas)
├── middleware/          # JWT auth, RBAC (require_admin, require_accountant)
└── errors/              # AppError tipado con ResponseError
migration/               # 17 migraciones SeaORM
```

**Principios de diseño:**
- `domain/` es puro: sin I/O, sin Actix, sin SeaORM. Solo Rust + `rust_decimal`.
- `api/` es delgada: deserializa → llama a services → serializa. Sin lógica de negocio.
- `services/` orquesta: conecta domain con BD, maneja transacciones, produce exportaciones.
- Las entidades de BD nunca se exponen directamente — siempre a través de DTOs.

---

## Endpoints

Base URL: `/billingVE/v1/`

### Facturas

| Método | Ruta                  | Rol mínimo  | Descripción        |
|--------|-----------------------|-------------|--------------------|
| `POST` | `/invoices`           | accountant  | Crear factura      |
| `GET`  | `/invoices`           | viewer      | Listar facturas    |
| `GET`  | `/invoices/{id}`      | viewer      | Detalle de factura |
| `POST` | `/invoices/{id}/void` | accountant  | Anular factura     |

### Notas de Crédito

| Método | Ruta                 | Rol mínimo | Descripción             |
|--------|----------------------|------------|-------------------------|
| `POST` | `/credit-notes`      | accountant | Crear nota de crédito   |
| `GET`  | `/credit-notes`      | viewer     | Listar notas de crédito |
| `GET`  | `/credit-notes/{id}` | viewer     | Detalle                 |

### Notas de Débito

| Método | Ruta                | Rol mínimo | Descripción            |
|--------|---------------------|------------|------------------------|
| `POST` | `/debit-notes`      | accountant | Crear nota de débito   |
| `GET`  | `/debit-notes`      | viewer     | Listar notas de débito |
| `GET`  | `/debit-notes/{id}` | viewer     | Detalle                |

### Guías de Despacho

| Método | Ruta                   | Rol mínimo | Descripción            |
|--------|------------------------|------------|------------------------|
| `POST` | `/delivery-notes`      | accountant | Crear guía de despacho |
| `GET`  | `/delivery-notes`      | viewer     | Listar guías           |
| `GET`  | `/delivery-notes/{id}` | viewer     | Detalle                |

### Clientes

| Método | Ruta            | Rol mínimo | Descripción        |
|--------|-----------------|------------|--------------------|
| `POST` | `/clients`      | accountant | Crear cliente      |
| `GET`  | `/clients`      | viewer     | Listar clientes    |
| `GET`  | `/clients/{id}` | viewer     | Detalle de cliente |
| `PUT`  | `/clients/{id}` | accountant | Actualizar cliente |

### Perfil de Empresa

| Método | Ruta       | Rol mínimo | Descripción                  |
|--------|------------|------------|------------------------------|
| `GET`  | `/company` | viewer     | Ver perfil fiscal del emisor |
| `PUT`  | `/company` | admin      | Actualizar perfil fiscal     |

### Libros Fiscales

| Método | Ruta                              | Rol mínimo | Descripción                           |
|--------|-----------------------------------|------------|---------------------------------------|
| `GET`  | `/books/purchases?period=2026-03` | viewer     | Consultar libro de compras            |
| `POST` | `/books/purchases`                | accountant | Registrar entrada en libro de compras |
| `GET`  | `/books/sales?period=2026-03`     | viewer     | Consultar libro de ventas             |
| `POST` | `/books/sales`                    | accountant | Registrar entrada en libro de ventas  |

### Retenciones IVA

| Método | Ruta                     | Rol mínimo | Descripción                           |
|--------|--------------------------|------------|---------------------------------------|
| `POST` | `/withholdings/iva`      | accountant | Crear comprobante de retención IVA    |
| `GET`  | `/withholdings/iva`      | viewer     | Listar retenciones IVA                |
| `GET`  | `/withholdings/iva/{id}` | viewer     | Detalle de retención IVA              |

### Retenciones ISLR

| Método | Ruta                      | Rol mínimo | Descripción                           |
|--------|---------------------------|------------|---------------------------------------|
| `POST` | `/withholdings/islr`      | accountant | Crear comprobante de retención ISLR   |
| `GET`  | `/withholdings/islr`      | viewer     | Listar retenciones ISLR               |
| `GET`  | `/withholdings/islr/{id}` | viewer     | Detalle de retención ISLR             |

### Reportes y Exportaciones

| Método | Ruta                                              | Auth    | Descripción                                  |
|--------|---------------------------------------------------|---------|----------------------------------------------|
| `GET`  | `/reports/invoices/{id}/pdf`                      | viewer  | Factura en PDF                               |
| `GET`  | `/reports/credit-notes/{id}/pdf`                  | viewer  | Nota de crédito en PDF                       |
| `GET`  | `/reports/debit-notes/{id}/pdf`                   | viewer  | Nota de débito en PDF                        |
| `GET`  | `/reports/withholdings/iva/{id}/pdf`              | viewer  | Comprobante retención IVA en PDF             |
| `GET`  | `/reports/withholdings/islr/{id}/pdf`             | viewer  | Comprobante ARC ISLR en PDF                  |
| `GET`  | `/reports/withholdings/iva/xml?period=202603-01`  | viewer  | XML retenciones IVA para SENIAT (quincenal)  |
| `GET`  | `/reports/withholdings/islr/txt?period=2026-03`   | viewer  | TXT retenciones ISLR para SENIAT (mensual)   |

### Health Check

| Método | Ruta      | Auth | Descripción         |
|--------|-----------|------|---------------------|
| `GET`  | `/health` | No   | Estado del servicio |

---

## Formato de Respuestas

**Éxito:**
```json
{
  "success": true,
  "data": { ... }
}
```

**Éxito paginado:**
```json
{
  "success": true,
  "data": [ ... ],
  "page": 1,
  "per_page": 25,
  "total": 150
}
```

**Error:**
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Factura no encontrada"
  }
}
```

**Parámetros de paginación:** `?page=1&per_page=25`
**Filtros por fecha:** `?from=2026-01-01&to=2026-01-31`

---

## Base de Datos

17 migraciones SeaORM que crean las siguientes tablas:

| Tabla                   | Descripción                                             |
|-------------------------|---------------------------------------------------------|
| `company_profiles`      | Perfil fiscal del emisor (razón social, RIF, domicilio) |
| `clients`               | Directorio de clientes/proveedores con validación RIF   |
| `exchange_rates`        | Tasas de cambio BCV/manual — snapshot inmutable por doc |
| `numbering_sequences`   | Secuencias de numeración de facturas por serie          |
| `control_number_ranges` | Rangos de números de control autorizados por SENIAT     |
| `invoices`              | Facturas (inmutables — nunca se editan ni eliminan)     |
| `invoice_items`         | Líneas de detalle de facturas                           |
| `credit_notes`          | Notas de crédito vinculadas a factura original          |
| `debit_notes`           | Notas de débito vinculadas a factura original           |
| `delivery_notes`        | Guías de despacho                                       |
| `tax_withholdings_iva`  | Retenciones de IVA — declaración quincenal              |
| `tax_withholdings_islr` | Retenciones de ISLR — declaración mensual               |
| `purchase_book_entries` | Entradas del libro de compras                           |
| `sales_book_entries`    | Entradas del libro de ventas                            |
| `audit_logs`            | Registro de auditoría de operaciones                    |

Las migraciones se ejecutan automáticamente al iniciar el servidor.

---

## Reglas Fiscales SENIAT

| Concepto               | Detalle                                                                 |
|------------------------|-------------------------------------------------------------------------|
| IVA general            | 16%                                                                     |
| IVA reducido           | 8%                                                                      |
| IVA lujo               | Hasta 31%                                                               |
| IVA exento             | 0%                                                                      |
| Retención IVA          | 75% o 100% del IVA — contribuyentes especiales, XML quincenal           |
| Retención ISLR         | Variable según actividad (Decreto 1.808) — TXT mensual                  |
| Número de control      | Formato `00-XXXXXXXX` — asignado por imprenta autorizada SENIAT         |
| RIF                    | Formato `[JVEGPC]-XXXXXXXX-X` con validación de dígito verificador      |
| Multi-moneda           | Bs. y USD — tasa BCV almacenada como snapshot inmutable por documento   |
| Inmutabilidad          | Las facturas emitidas nunca se editan — se anulan con nota de crédito   |
| Consumidores finales   | Obligatorio imprimir "SIN DERECHO A CRÉDITO FISCAL" si no tienen RIF   |
| Retención de datos     | Mínimo 10 años (requisito legal SENIAT)                                 |

## Normativa de Referencia

| Documento             | Descripción                             |
|-----------------------|-----------------------------------------|
| PA SNAT/2011/0071     | Normas generales de emisión de facturas |
| PA SNAT/2024/000102   | Facturación digital obligatoria         |
| PA SNAT/2015/0049     | Retenciones de IVA                      |
| Decreto 1.808         | Retenciones de ISLR                     |

---

## Licencia

Uso privado. Todos los derechos reservados.
