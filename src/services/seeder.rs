//! Seeder: inserta datos iniciales obligatorios al primer arranque.
//!
//! Lee la configuración de variables de entorno (SeedConfig) y crea:
//! - Perfil de empresa (company_profiles)
//! - Secuencias de numeración (numbering_sequences) para cada tipo de documento
//! - Rango de números de control (control_number_ranges)
//!
//! Es idempotente: si el RIF de la empresa ya existe, no inserta nada.

use chrono::Utc;
use log::info;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::config::SeedConfig;
use crate::entities::{company_profiles, control_number_ranges, numbering_sequences};
use crate::errors::AppError;

/// UUID fijo para el usuario "system" que crea los datos seed.
const SYSTEM_USER_ID: &str = "00000000-0000-0000-0000-000000000000";

/// Ejecuta el seeder si hay configuración seed disponible.
pub async fn run_seed(db: &DatabaseConnection, config: &SeedConfig) -> Result<(), AppError> {
    let system_user: Uuid = SYSTEM_USER_ID.parse().map_err(|e| {
        AppError::Internal(format!(
            "Constante SYSTEM_USER_ID inválida '{}': {}",
            SYSTEM_USER_ID, e
        ))
    })?;

    // Verificar si ya existe la empresa con ese RIF
    let existing = company_profiles::Entity::find()
        .filter(company_profiles::Column::Rif.eq(&config.company_rif))
        .one(db)
        .await?;

    if existing.is_some() {
        info!(
            "Seed: empresa con RIF {} ya existe, saltando seeder.",
            config.company_rif
        );
        return Ok(());
    }

    info!(
        "Seed: creando empresa '{}' (RIF: {})...",
        config.company_razon_social, config.company_rif
    );

    let now = Utc::now().into();
    let company_id = Uuid::new_v4();

    // 1. Crear perfil de empresa
    let company = company_profiles::ActiveModel {
        id: Set(company_id),
        razon_social: Set(config.company_razon_social.clone()),
        nombre_comercial: Set(config.company_nombre_comercial.clone()),
        rif: Set(config.company_rif.clone()),
        domicilio_fiscal: Set(config.company_domicilio_fiscal.clone()),
        telefono: Set(config.company_telefono.clone()),
        email: Set(config.company_email.clone()),
        es_contribuyente_especial: Set(config.company_es_contribuyente_especial),
        nro_contribuyente_especial: Set(config.company_nro_contribuyente_especial.clone()),
        is_active: Set(true),
        created_by: Set(system_user),
        created_at: Set(now),
        updated_at: Set(now),
    };
    company.insert(db).await?;
    info!("Seed: empresa creada con ID {}", company_id);

    // 2. Crear secuencias de numeración para cada tipo de documento
    let sequence_types = [
        ("INVOICE", Some("FAC-")),
        ("CREDIT_NOTE", Some("NC-")),
        ("DEBIT_NOTE", Some("ND-")),
        ("DELIVERY_NOTE", Some("GD-")),
    ];

    for (seq_type, prefix) in sequence_types {
        let seq = numbering_sequences::ActiveModel {
            id: Set(Uuid::new_v4()),
            sequence_type: Set(seq_type.to_string()),
            prefix: Set(prefix.map(|s| s.to_string())),
            current_value: Set(0),
            min_value: Set(1),
            max_value: Set(None),
            is_active: Set(true),
            company_profile_id: Set(company_id),
            created_by: Set(system_user),
            created_at: Set(now),
            updated_at: Set(now),
        };
        seq.insert(db).await?;
        info!(
            "Seed: secuencia {} creada para empresa {}",
            seq_type, company_id
        );
    }

    // 3. Crear rango de números de control
    let control_range = control_number_ranges::ActiveModel {
        id: Set(Uuid::new_v4()),
        company_profile_id: Set(company_id),
        prefix: Set(config.control_prefix.clone()),
        range_from: Set(config.control_range_from),
        range_to: Set(config.control_range_to),
        current_value: Set(config.control_range_from - 1),
        is_active: Set(true),
        imprenta_autorizada: Set(config.control_imprenta.clone()),
        fecha_autorizacion: Set(None),
        created_by: Set(system_user),
        created_at: Set(now),
        updated_at: Set(now),
    };
    control_range.insert(db).await?;
    info!(
        "Seed: rango de control {}-{:08} a {}-{:08} creado",
        config.control_prefix, config.control_range_from, config.control_prefix, config.control_range_to
    );

    info!("Seed: completado exitosamente.");
    Ok(())
}
