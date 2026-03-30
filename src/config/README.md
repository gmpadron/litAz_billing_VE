# config/ — Configuración de la Aplicación

Módulo de configuración que carga y valida variables de entorno al arranque. Expone `Settings` y la función `establish_connection`.

## Archivos

| Archivo         | Descripción                                                                    |
|-----------------|--------------------------------------------------------------------------------|
| `mod.rs`        | Re-exporta `Settings`, `SeedConfig` y `establish_connection`.                  |
| `settings.rs`   | Struct `Settings` con toda la configuración. Carga desde `std::env` al iniciar. |
| `database.rs`   | Función `establish_connection(&database_url)` — crea el pool SeaORM.           |

## Settings — Campos

| Campo           | Tipo              | Variable de entorno    | Default      | Descripción                                         |
|-----------------|-------------------|------------------------|--------------|-----------------------------------------------------|
| `database_url`  | `String`          | `DATABASE_URL`         | requerido    | URL de conexión PostgreSQL.                         |
| `jwt_secret`    | `String`          | `JWT_SECRET`           | requerido    | Secret HS256 compartido con `litAz_auth_service`.   |
| `jwt_issuer`    | `Option<String>`  | `JWT_ISSUER`           | `None`       | Si se define, valida el campo `iss` del JWT.        |
| `jwt_audience`  | `Option<String>`  | `JWT_AUDIENCE`         | `None`       | Si se define, valida el campo `aud` del JWT.        |
| `server_host`   | `String`          | `SERVER_HOST`          | `0.0.0.0`    | Host en el que escucha el servidor.                 |
| `server_port`   | `u16`             | `SERVER_PORT`          | `8080`       | Puerto del servidor.                                |
| `cors_origins`  | `Vec<String>`     | `CORS_ORIGINS`         | `[]`         | Orígenes permitidos (separados por coma).           |
| `seed`          | `Option<SeedConfig>` | `SEED_*`            | `None`       | Si las variables `SEED_COMPANY_*` están definidas, crea el perfil de empresa en el primer arranque. |

## SeedConfig — Campos

El seed solo se ejecuta si `SEED_COMPANY_RAZON_SOCIAL`, `SEED_COMPANY_RIF`, `SEED_COMPANY_DOMICILIO_FISCAL` y `SEED_COMPANY_TELEFONO` están presentes en el entorno.

| Campo                              | Variable de entorno                         | Descripción                                    |
|------------------------------------|---------------------------------------------|------------------------------------------------|
| `company_razon_social`             | `SEED_COMPANY_RAZON_SOCIAL`                 | Razón social de la empresa (requerido).        |
| `company_nombre_comercial`         | `SEED_COMPANY_NOMBRE_COMERCIAL`             | Nombre comercial (opcional).                   |
| `company_rif`                      | `SEED_COMPANY_RIF`                          | RIF en formato `J-XXXXXXXX-X` (requerido).     |
| `company_domicilio_fiscal`         | `SEED_COMPANY_DOMICILIO_FISCAL`             | Dirección fiscal completa (requerido).         |
| `company_telefono`                 | `SEED_COMPANY_TELEFONO`                     | Teléfono de contacto (requerido).              |
| `company_email`                    | `SEED_COMPANY_EMAIL`                        | Email de facturación (opcional).               |
| `company_es_contribuyente_especial`| `SEED_COMPANY_ES_CONTRIBUYENTE_ESPECIAL`    | `true`/`false` — contribuyente especial SENIAT.|
| `company_nro_contribuyente_especial`| `SEED_COMPANY_NRO_CONTRIBUYENTE_ESPECIAL`  | Número de contribuyente especial (opcional).   |
| `control_prefix`                   | `SEED_CONTROL_PREFIX`                       | Prefijo del número de control (`00`).          |
| `control_range_from`               | `SEED_CONTROL_RANGE_FROM`                   | Inicio del rango autorizado (default: `1`).    |
| `control_range_to`                 | `SEED_CONTROL_RANGE_TO`                     | Fin del rango autorizado (default: `99999999`).|
| `control_imprenta`                 | `SEED_CONTROL_IMPRENTA`                     | Nombre de la imprenta autorizada SENIAT.       |

## Uso en main.rs

```rust
let settings = Settings::from_env().expect("Failed to load settings");

let db = config::establish_connection(&settings.database_url).await?;

// El seed se ejecuta solo si las variables SEED_* están definidas
if let Some(ref seed_config) = settings.seed {
    services::seeder::run_seed(&db, seed_config).await?;
}
```

## Variables de entorno críticas

| Variable        | Descripción                                                                |
|-----------------|----------------------------------------------------------------------------|
| `DATABASE_URL`  | Debe comenzar con `postgres://` o `postgresql://`.                         |
| `JWT_SECRET`    | Debe ser el mismo valor que `JWT_ACCESS_SECRET` en `litAz_auth_service`.   |
| `RUST_LOG`      | Nivel de logging: `error`, `warn`, `info`, `debug`, `trace`.               |

## CORS en producción

Si `CORS_ORIGINS` está vacío en un build de release, solo se permite `http://localhost:3000`. En builds de debug con `CORS_ORIGINS` vacío, se permiten todos los orígenes.

Para producción, definir explícitamente:
```env
CORS_ORIGINS=https://miapp.com,https://admin.miapp.com
```
