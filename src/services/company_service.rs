//! Servicio del perfil fiscal de empresa — soporte multi-empresa.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::domain::numbering::rif::Rif;
use crate::dto::company_dto::{
    CompanyListItem, CompanyResponse, CreateCompanyRequest, UpdateCompanyRequest,
};
use crate::entities::company_profiles;
use crate::errors::AppError;

/// Lista todas las empresas (activas e inactivas).
pub async fn list_companies(db: &DatabaseConnection) -> Result<Vec<CompanyListItem>, AppError> {
    let companies = company_profiles::Entity::find()
        .order_by_asc(company_profiles::Column::CreatedAt)
        .all(db)
        .await?;

    Ok(companies.into_iter().map(model_to_list_item).collect())
}

/// Obtiene una empresa por ID.
pub async fn get_company_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<CompanyResponse, AppError> {
    let company = company_profiles::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Empresa con ID {} no encontrada.", id)))?;

    Ok(model_to_response(company))
}

/// Crea una nueva empresa.
pub async fn create_company(
    db: &DatabaseConnection,
    dto: CreateCompanyRequest,
    user_id: Uuid,
) -> Result<CompanyResponse, AppError> {
    Rif::parse(&dto.rif)
        .map_err(|e| AppError::Validation(format!("RIF inválido: {}", e)))?;

    let existing = company_profiles::Entity::find()
        .filter(company_profiles::Column::Rif.eq(&dto.rif))
        .one(db)
        .await?;
    if existing.is_some() {
        return Err(AppError::BadRequest(format!(
            "Ya existe una empresa con RIF {}.",
            dto.rif
        )));
    }

    let now = Utc::now().into();
    let id = Uuid::new_v4();

    let model = company_profiles::ActiveModel {
        id: Set(id),
        razon_social: Set(dto.business_name),
        nombre_comercial: Set(dto.trade_name),
        rif: Set(dto.rif),
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

    let inserted = model.insert(db).await?;
    Ok(model_to_response(inserted))
}

/// Actualiza una empresa existente por ID.
pub async fn update_company(
    db: &DatabaseConnection,
    id: Uuid,
    dto: UpdateCompanyRequest,
    _user_id: Uuid,
) -> Result<CompanyResponse, AppError> {
    let company = company_profiles::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Empresa con ID {} no encontrada.", id)))?;

    let now = Utc::now().into();
    let mut active: company_profiles::ActiveModel = company.into();

    if let Some(ref name) = dto.business_name {
        active.razon_social = Set(name.clone());
    }
    if let Some(ref trade_name) = dto.trade_name {
        active.nombre_comercial = Set(Some(trade_name.clone()));
    }
    if let Some(ref rif) = dto.rif {
        Rif::parse(rif).map_err(|e| AppError::Validation(format!("RIF inválido: {}", e)))?;
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

/// Desactiva una empresa (soft delete). No elimina datos históricos.
pub async fn deactivate_company(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
    let company = company_profiles::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Empresa con ID {} no encontrada.", id)))?;

    let mut active: company_profiles::ActiveModel = company.into();
    active.is_active = Set(false);
    active.updated_at = Set(Utc::now().into());
    active.update(db).await?;
    Ok(())
}

// ── Mappers ───────────────────────────────────────────────────────────────────

fn model_to_list_item(m: company_profiles::Model) -> CompanyListItem {
    CompanyListItem {
        id: m.id,
        business_name: m.razon_social,
        trade_name: m.nombre_comercial,
        rif: m.rif,
        is_special_taxpayer: m.es_contribuyente_especial,
        is_active: m.is_active,
        updated_at: m.updated_at.with_timezone(&Utc),
    }
}

fn model_to_response(m: company_profiles::Model) -> CompanyResponse {
    CompanyResponse {
        id: m.id,
        business_name: m.razon_social,
        trade_name: m.nombre_comercial,
        rif: m.rif,
        fiscal_address: m.domicilio_fiscal,
        phone: Some(m.telefono).filter(|s| !s.is_empty()),
        email: m.email,
        is_special_taxpayer: m.es_contribuyente_especial,
        special_taxpayer_resolution: m.nro_contribuyente_especial,
        is_active: m.is_active,
        updated_at: m.updated_at.with_timezone(&Utc),
    }
}
