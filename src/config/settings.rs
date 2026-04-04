/// Configuración global de la aplicación, cargada desde variables de entorno.
#[derive(Debug, Clone)]
pub struct Settings {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_issuer: Option<String>,
    pub jwt_audience: Option<String>,
    pub server_host: String,
    pub server_port: u16,
    pub cors_origins: Vec<String>,
    pub seed: Option<SeedConfig>,
}

/// Datos para el seeder inicial (perfil de empresa + secuencias).
#[derive(Debug, Clone)]
pub struct SeedConfig {
    pub company_razon_social: String,
    pub company_nombre_comercial: Option<String>,
    pub company_rif: String,
    pub company_domicilio_fiscal: String,
    pub company_telefono: String,
    pub company_email: Option<String>,
    pub company_es_contribuyente_especial: bool,
    pub company_nro_contribuyente_especial: Option<String>,
    pub control_prefix: String,
    pub control_range_from: i64,
    pub control_range_to: i64,
    pub control_imprenta: Option<String>,
}

impl Settings {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        let seed = Self::load_seed_config();

        let cors_origins = std::env::var("CORS_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let jwt_secret = std::env::var("JWT_SECRET")?;

        // Un secreto JWT corto permite ataques de fuerza bruta sobre el HMAC-SHA256.
        // 32 caracteres = 256 bits mínimo, equivalente a la longitud del hash SHA-256.
        if jwt_secret.len() < 32 {
            panic!(
                "JWT_SECRET debe tener al menos 32 caracteres (tiene {}). \
                 Use un secreto generado aleatoriamente, por ejemplo: \
                 openssl rand -hex 32",
                jwt_secret.len()
            );
        }

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            jwt_secret,
            jwt_issuer: std::env::var("JWT_ISSUER").ok().filter(|s| !s.is_empty()),
            jwt_audience: std::env::var("JWT_AUDIENCE").ok().filter(|s| !s.is_empty()),
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid u16"),
            cors_origins,
            seed,
        })
    }

    fn load_seed_config() -> Option<SeedConfig> {
        let razon_social = std::env::var("SEED_COMPANY_RAZON_SOCIAL").ok()?;
        let rif = std::env::var("SEED_COMPANY_RIF").ok()?;
        let domicilio = std::env::var("SEED_COMPANY_DOMICILIO_FISCAL").ok()?;
        let telefono = std::env::var("SEED_COMPANY_TELEFONO").ok()?;

        Some(SeedConfig {
            company_razon_social: razon_social,
            company_nombre_comercial: std::env::var("SEED_COMPANY_NOMBRE_COMERCIAL")
                .ok()
                .filter(|s| !s.is_empty()),
            company_rif: rif,
            company_domicilio_fiscal: domicilio,
            company_telefono: telefono,
            company_email: std::env::var("SEED_COMPANY_EMAIL")
                .ok()
                .filter(|s| !s.is_empty()),
            company_es_contribuyente_especial: std::env::var(
                "SEED_COMPANY_ES_CONTRIBUYENTE_ESPECIAL",
            )
            .ok()
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false),
            company_nro_contribuyente_especial: std::env::var(
                "SEED_COMPANY_NRO_CONTRIBUYENTE_ESPECIAL",
            )
            .ok()
            .filter(|s| !s.is_empty()),
            control_prefix: std::env::var("SEED_CONTROL_PREFIX").unwrap_or_else(|_| "00".into()),
            control_range_from: std::env::var("SEED_CONTROL_RANGE_FROM")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            control_range_to: std::env::var("SEED_CONTROL_RANGE_TO")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(99_999_999),
            control_imprenta: std::env::var("SEED_CONTROL_IMPRENTA")
                .ok()
                .filter(|s| !s.is_empty()),
        })
    }
}
