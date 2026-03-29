//! Servicio del perfil fiscal de la empresa emisora: persistencia real.

use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::domain::numbering::rif::Rif;
use crate::dto::company_dto::{CompanyResponse, UpdateCompanyRequest};
use crate::entities::company_profiles;
use crate::errors::AppError;

/// Obtiene el primer perfil fiscal de empresa activo.
pub async fn get_company(db: &DatabaseConnection) -> Result<CompanyResponse, AppError> {
    let company = company_profiles::Entity::find()
        .filter(company_profiles::Column::IsActive.eq(true))
        .one(db)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(
                "Perfil de empresa no configurado. Use PUT /billingVE/v1/company para configurarlo."
                    .to_string(),
            )
        })?;

    Ok(model_to_response(company))
}

/// Actualiza el perfil fiscal de la empresa (upsert).
pub async fn update_company(
    db: &DatabaseConnection,
    dto: UpdateCompanyRequest,
    user_id: Uuid,
) -> Result<CompanyResponse, AppError> {
    // Buscar perfil existente
    let existing = company_profiles::Entity::find()
        .filter(company_profiles::Column::IsActive.eq(true))
        .one(db)
        .await?;

    let now = Utc::now().into();

    match existing {
        Some(company) => {
            let mut active: company_profiles::ActiveModel = company.into();

            if let Some(ref name) = dto.business_name {
                active.razon_social = Set(name.clone());
            }
            if let Some(ref trade_name) = dto.trade_name {
                active.nombre_comercial = Set(Some(trade_name.clone()));
            }
            if let Some(ref rif) = dto.rif {
                Rif::parse(rif).map_err(|e| {
                    AppError::Validation(format!("RIF inválido: {}", e))
                })?;
                active.rif = Set(rif.clone());
            }
            if let Some(ref address) = dto.fiscal_address {
                active.domicilio_fiscal = Set(address.clone());
            }
            if let Some(ref phone) = dto.phone {
                active.telefono = Set(phone.clone());
            }
            if let Some(ref email) = dto.email {
                active.email = Set(Some(email.clone()));
            }
            if let Some(is_special) = dto.is_special_taxpayer {
                active.es_contribuyente_especial = Set(is_special);
            }
            if let Some(ref resolution) = dto.special_taxpayer_resolution {
                active.nro_contribuyente_especial = Set(Some(resolution.clone()));
            }
            active.updated_at = Set(now);

            let updated = active.update(db).await?;
            Ok(model_to_response(updated))
        }
        None => {
            // Crear nuevo perfil
            let business_name = dto.business_name.ok_or_else(|| {
                AppError::BadRequest(
                    "Se requiere la razón social para crear el perfil".to_string(),
                )
            })?;
            let rif = dto.rif.ok_or_else(|| {
                AppError::BadRequest("Se requiere el RIF para crear el perfil".to_string())
            })?;
            Rif::parse(&rif).map_err(|e| {
                AppError::Validation(format!("RIF inválido: {}", e))
            })?;

            let id = Uuid::new_v4();
            let company = company_profiles::ActiveModel {
                id: Set(id),
                razon_social: Set(business_name),
                nombre_comercial: Set(dto.trade_name),
                rif: Set(rif),
                domicilio_fiscal: Set(dto.fiscal_address.unwrap_or_default()),
                telefono: Set(dto.phone.unwrap_or_default()),
                email: Set(dto.email),
                es_contribuyente_especial: Set(dto.is_special_taxpayer.unwrap_or(false)),
                nro_contribuyente_especial: Set(dto.special_taxpayer_resolution),
                is_active: Set(true),
                created_by: Set(user_id),
                created_at: Set(now),
                updated_at: Set(now),
            };

            let inserted = company.insert(db).await?;
            Ok(model_to_response(inserted))
        }
    }
}

fn model_to_response(m: company_profiles::Model) -> CompanyResponse {
    CompanyResponse {
        id: m.id,
        business_name: m.razon_social,
        trade_name: m.nombre_comercial,
        rif: m.rif,
        fiscal_address: m.domicilio_fiscal,
        phone: Some(m.telefono),
        email: m.email,
        is_special_taxpayer: m.es_contribuyente_especial,
        special_taxpayer_resolution: m.nro_contribuyente_especial,
        updated_at: m.updated_at.with_timezone(&Utc),
    }
}
