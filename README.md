# litAz Billing Core

API REST de facturación fiscal para Venezuela, conforme a las normativas del SENIAT. Backend en Rust con Actix Web y PostgreSQL.

## Stack Tecnológico

| Componente         | Tecnología                                                         |
| ------------------ | ------------------------------------------------------------------ |
| Lenguaje           | Rust (Edition 2024)                                                |
| Framework HTTP     | Actix Web 4                                                        |
| ORM                | SeaORM (migraciones automáticas al iniciar)                        |
| Base de datos      | PostgreSQL 16+                                                     |
| Precisión numérica | `rust_decimal` (128-bit) para todo cálculo monetario               |
| Autenticación      | JWT (`jsonwebtoken`) — tokens emitidos por servicio externo NestJS |
| PDF                | `genpdf`                                                           |
| XML/TXT            | `quick-xml` para retenciones IVA, TXT para ISLR                    |
| Validación         | `validator` crate                                                  |

## Requisitos

- Rust 1.85+ (edition 2024)
- PostgreSQL 16+
- Servicio de autenticación externo ([litAz_auth_service](https://github.com/tu-org/litAz_auth_service)) emitiendo JWT

## Instalación

```bash

# Configurar variables de entorno
cp .env.example .env
# Editar .env con tus valores (ver sección Variables de Entorno)

# Crear base de datos
createdb billing_core

# Compilar y ejecutar (las migraciones corren automáticamente)
cargo run
```

## Variables de Entorno

.env.example

Si se definen las variables `SEED_COMPANY_*` y `SEED_CONTROL_*`, al iniciar por primera vez el servidor creará automáticamente el perfil de empresa, la secuencia de numeración de facturas y el rango de números de control.

## Comandos

```bash
cargo run                    # Iniciar servidor
cargo test                   # Ejecutar tests
cargo fmt                    # Formatear código
cargo clippy                 # Linter
```

## Arquitectura

```
src/
├── main.rs                  # Entry point, configuración del servidor
├── config/                  # Settings, conexión a BD
├── api/                     # Handlers HTTP (capa delgada, sin lógica de negocio)
├── domain/                  # Lógica de negocio pura (sin dependencias de framework)
│   ├── invoice/             # Builder de facturas, estados, validación
│   ├── numbering/           # RIF, números de control, secuencias
│   ├── tax/                 # Cálculos IVA e ISLR
│   ├── withholding/         # Cálculos de retenciones
│   ├── books/               # Libros de compras y ventas
│   └── currency/            # Tasa de cambio BCV
├── services/                # Orquestación entre domain y BD
├── dto/                     # Request/Response structs
├── entities/                # Modelos SeaORM (15 tablas)
├── middleware/              # Autenticación JWT, RBAC
└── errors/                  # Tipos de error tipados
migration/                   # 17 migraciones SeaORM
```

**Principios:**

- `domain/` es puro: sin I/O, sin frameworks. Solo Rust + `rust_decimal`.
- `api/` es delgada: deserializa, llama a services, serializa.
- `services/` orquesta: conecta domain con BD, maneja transacciones.
- Las entidades de BD nunca se exponen directamente — siempre a través de DTOs.

## Autenticación y Autorización

La autenticación es **externa**. Este servicio solo valida tokens JWT emitidos por `litAz_auth_service` (NestJS).

**Payload JWT esperado:**

```json
{
  "sub": "user-uuid",
  "email": "user@example.com",
  "roles": ["admin"],
  "permissions": ["invoices:create", "invoices:read"],
  "sessionId": "session-uuid",
  "deviceId": "fingerprint",
  "twoFactorVerified": true,
  "iss": "ecommerce-api",
  "aud": "ecommerce-frontend",
  "iat": 1234567890,
  "exp": 1234567890
}
```

**Roles:**
| Rol | Permisos |
|---|---|
| `admin` | Acceso total. Puede modificar perfil de empresa. |
| `accountant` | Crear/anular facturas, notas, retenciones, libros. |
| `viewer` | Solo lectura en todos los endpoints. |

**Modo desarrollo:** En builds de debug (`cargo run`), se puede enviar el header `X-User-Id` sin token JWT para pruebas locales.

## Endpoints

Base URL: `/billingVE/v1/`

### Facturas

| Método | Ruta                  | Rol mínimo | Descripción        |
| ------ | --------------------- | ---------- | ------------------ |
| `POST` | `/invoices`           | accountant | Crear factura      |
| `GET`  | `/invoices`           | viewer     | Listar facturas    |
| `GET`  | `/invoices/{id}`      | viewer     | Detalle de factura |
| `POST` | `/invoices/{id}/void` | accountant | Anular factura     |

### Notas de Crédito

| Método | Ruta                 | Rol mínimo | Descripción             |
| ------ | -------------------- | ---------- | ----------------------- |
| `POST` | `/credit-notes`      | accountant | Crear nota de crédito   |
| `GET`  | `/credit-notes`      | viewer     | Listar notas de crédito |
| `GET`  | `/credit-notes/{id}` | viewer     | Detalle                 |

### Notas de Débito

| Método | Ruta                | Rol mínimo | Descripción            |
| ------ | ------------------- | ---------- | ---------------------- |
| `POST` | `/debit-notes`      | accountant | Crear nota de débito   |
| `GET`  | `/debit-notes`      | viewer     | Listar notas de débito |
| `GET`  | `/debit-notes/{id}` | viewer     | Detalle                |

### Guías de Despacho

| Método | Ruta                   | Rol mínimo | Descripción            |
| ------ | ---------------------- | ---------- | ---------------------- |
| `POST` | `/delivery-notes`      | accountant | Crear guía de despacho |
| `GET`  | `/delivery-notes`      | viewer     | Listar guías           |
| `GET`  | `/delivery-notes/{id}` | viewer     | Detalle                |

### Clientes

| Método | Ruta            | Rol mínimo | Descripción        |
| ------ | --------------- | ---------- | ------------------ |
| `POST` | `/clients`      | accountant | Crear cliente      |
| `GET`  | `/clients`      | viewer     | Listar clientes    |
| `GET`  | `/clients/{id}` | viewer     | Detalle            |
| `PUT`  | `/clients/{id}` | accountant | Actualizar cliente |

### Perfil de Empresa

| Método | Ruta       | Rol mínimo | Descripción       |
| ------ | ---------- | ---------- | ----------------- |
| `GET`  | `/company` | viewer     | Ver perfil fiscal |
| `PUT`  | `/company` | admin      | Actualizar perfil |

### Libros Fiscales

| Método | Ruta                              | Rol mínimo | Descripción                           |
| ------ | --------------------------------- | ---------- | ------------------------------------- |
| `POST` | `/books/purchases`                | accountant | Registrar entrada en libro de compras |
| `GET`  | `/books/purchases?period=2026-03` | viewer     | Consultar libro de compras            |
| `POST` | `/books/sales`                    | accountant | Registrar entrada en libro de ventas  |
| `GET`  | `/books/sales?period=2026-03`     | viewer     | Consultar libro de ventas             |

### Retenciones IVA

| Método | Ruta                     | Rol mínimo | Descripción         |
| ------ | ------------------------ | ---------- | ------------------- |
| `POST` | `/withholdings/iva`      | accountant | Crear retención IVA |
| `GET`  | `/withholdings/iva`      | viewer     | Listar retenciones  |
| `GET`  | `/withholdings/iva/{id}` | viewer     | Detalle             |

### Retenciones ISLR

| Método | Ruta                      | Rol mínimo | Descripción          |
| ------ | ------------------------- | ---------- | -------------------- |
| `POST` | `/withholdings/islr`      | accountant | Crear retención ISLR |
| `GET`  | `/withholdings/islr`      | viewer     | Listar retenciones   |
| `GET`  | `/withholdings/islr/{id}` | viewer     | Detalle              |

### Reportes y Exportaciones

| Método | Ruta                                             | Descripción                                 |
| ------ | ------------------------------------------------ | ------------------------------------------- |
| `GET`  | `/reports/invoices/{id}/pdf`                     | Factura en PDF                              |
| `GET`  | `/reports/credit-notes/{id}/pdf`                 | Nota de crédito en PDF                      |
| `GET`  | `/reports/debit-notes/{id}/pdf`                  | Nota de débito en PDF                       |
| `GET`  | `/reports/withholdings/iva/{id}/pdf`             | Comprobante retención IVA (PDF)             |
| `GET`  | `/reports/withholdings/islr/{id}/pdf`            | Comprobante ARC ISLR (PDF)                  |
| `GET`  | `/reports/withholdings/iva/xml?period=202603-01` | XML retenciones IVA para SENIAT (quincenal) |
| `GET`  | `/reports/withholdings/islr/txt?period=2026-03`  | TXT retenciones ISLR para SENIAT (mensual)  |

### Health Check

| Método | Ruta      | Auth | Descripción         |
| ------ | --------- | ---- | ------------------- |
| `GET`  | `/health` | No   | Estado del servicio |

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
    "code": "INVOICE_NOT_FOUND",
    "message": "Factura no encontrada"
  }
}
```

**Parámetros de paginación:** `?page=1&per_page=25`
**Filtros por fecha:** `?from=2026-01-01&to=2026-01-31`

## Base de Datos

17 migraciones que crean las siguientes tablas:

| Tabla                   | Descripción                                             |
| ----------------------- | ------------------------------------------------------- |
| `company_profiles`      | Perfil fiscal del emisor (razón social, RIF, domicilio) |
| `clients`               | Directorio de clientes/proveedores                      |
| `invoices`              | Facturas (inmutables, nunca se eliminan)                |
| `invoice_items`         | Líneas de detalle de facturas                           |
| `credit_notes`          | Notas de crédito                                        |
| `debit_notes`           | Notas de débito                                         |
| `delivery_notes`        | Guías de despacho                                       |
| `tax_withholdings_iva`  | Retenciones de IVA (quincenal)                          |
| `tax_withholdings_islr` | Retenciones de ISLR (mensual)                           |
| `purchase_book_entries` | Libro de compras                                        |
| `sales_book_entries`    | Libro de ventas                                         |
| `exchange_rates`        | Tasas de cambio (BCV/manual)                            |
| `numbering_sequences`   | Secuencias de numeración de facturas                    |
| `control_number_ranges` | Rangos de números de control SENIAT                     |
| `audit_logs`            | Registro de auditoría                                   |

Las migraciones se ejecutan automáticamente al iniciar el servidor.

## Reglas Fiscales SENIAT

- **IVA:** 16% (general), 8% (reducida), hasta 31% (lujo), 0% (exento)
- **Retención IVA:** 75% o 100% del IVA — contribuyentes especiales, declaración quincenal en XML
- **Retención ISLR:** variable según actividad (Decreto 1.808), declaración mensual en TXT
- **Número de control:** formato `00-XXXXXXXX`, asignado por imprenta autorizada SENIAT
- **RIF:** formato `[JVEGPC]-XXXXXXXX-X` con validación de dígito verificador
- **Multi-moneda:** Bs. y USD, tasa BCV del día de la operación almacenada como snapshot inmutable
- **Inmutabilidad:** las facturas emitidas nunca se editan ni eliminan — se anulan y se emite nota de crédito
- **"SIN DERECHO A CRÉDITO FISCAL":** obligatorio en facturas a consumidores finales sin RIF
- **Retención de datos:** mínimo 10 años (requisito legal)

## Normativa de Referencia

| Documento           | Descripción                             |
| ------------------- | --------------------------------------- |
| PA SNAT/2011/0071   | Normas generales de emisión de facturas |
| PA SNAT/2024/000102 | Facturación digital obligatoria         |
| PA SNAT/2015/0049   | Retenciones de IVA                      |
| Decreto 1.808       | Retenciones de ISLR                     |

## Licencia

Uso privado. Todos los derechos reservados.
