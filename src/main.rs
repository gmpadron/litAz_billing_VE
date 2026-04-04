#![allow(dead_code)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::too_many_arguments)]

mod api;
mod config;
mod domain;
mod dto;
mod entities;
mod errors;
mod middleware;
mod services;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, web};
use config::Settings;
use log::info;
use middleware::{CompanyMiddleware, JwtConfig, JwtMiddleware};
use std::sync::Arc;
use migration::{Migrator, MigratorTrait};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let settings = Settings::from_env().expect("Failed to load settings from environment");

    let db = config::establish_connection(&settings.database_url)
        .await
        .expect("Failed to connect to database");

    info!("Running database migrations...");
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    info!("Migrations complete.");

    // Ejecutar seeder si hay configuración seed en el .env
    if let Some(ref seed_config) = settings.seed {
        info!("Running seed data...");
        services::seeder::run_seed(&db, seed_config)
            .await
            .expect("Failed to run seeder");
    }

    info!(
        "Starting server at {}:{}",
        settings.server_host, settings.server_port
    );

    let jwt_config = JwtConfig {
        secret: settings.jwt_secret.clone(),
        issuer: settings.jwt_issuer.clone(),
        audience: settings.jwt_audience.clone(),
    };
    let cors_origins = settings.cors_origins.clone();
    let db_arc = Arc::new(db.clone());

    HttpServer::new(move || {
        // Configurar CORS
        let cors = build_cors(&cors_origins);

        App::new()
            .wrap(cors)
            .wrap(JwtMiddleware::new(jwt_config.clone()))
            .wrap(CompanyMiddleware::new(db_arc.clone()))
            .app_data(web::Data::new(db.clone()))
            // Payload JSON max 256 KB (protección contra payloads gigantes)
            .app_data(web::JsonConfig::default().limit(262_144))
            // Health check (público, no requiere JWT)
            .route("/health", web::get().to(health_check))
            .configure(api::configure)
    })
    .bind((settings.server_host.as_str(), settings.server_port))?
    .run()
    .await?;

    Ok(())
}

/// Health check endpoint — público, sin autenticación.
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "billing_core"
    }))
}

/// Construye la configuración CORS.
fn build_cors(origins: &[String]) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::AUTHORIZATION,
            actix_web::http::header::ACCEPT,
            actix_web::http::header::HeaderName::from_static("x-user-id"),
            actix_web::http::header::HeaderName::from_static("x-device-fingerprint"),
            actix_web::http::header::HeaderName::from_static("x-company-id"),
        ])
        .max_age(600);

    if origins.is_empty() {
        // Sin orígenes configurados: permitir solo localhost en desarrollo
        #[cfg(debug_assertions)]
        {
            cors = cors.allow_any_origin();
        }
        #[cfg(not(debug_assertions))]
        {
            // En producción sin CORS_ORIGINS configurado, no permitir ningún origen
            cors = cors.allowed_origin("http://localhost:3000");
        }
    } else {
        for origin in origins {
            cors = cors.allowed_origin(origin);
        }
    }

    cors
}
