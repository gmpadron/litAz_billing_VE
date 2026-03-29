//! Servicio de clientes: CRUD real contra base de datos.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

use crate::domain::numbering::rif::Rif;
use crate::dto::PaginatedResponse;
use crate::dto::client_dto::{
    ClientFilters, ClientListResponse, ClientResponse, CreateClientRequest, UpdateClientRequest,
};
use crate::entities::clients;
use crate::errors::AppError;

/// Crea un nuevo cliente.
pub async fn create_client(
    db: &DatabaseConnection,
    dto: CreateClientRequest,
    user_id: Uuid,
) -> Result<ClientResponse, AppError> {
    // Validar formato de RIF si se proporciona
    if let Some(ref rif) = dto.rif {
        Rif::parse(rif).map_err(|e| {
            AppError::Validation(format!("RIF del cliente inválido: {}", e))
        })?;

        let existing = clients::Entity::find()
            .filter(clients::Column::Rif.eq(rif.as_str()))
            .one(db)
            .await?;
        if existing.is_some() {
            return Err(AppError::BadRequest(format!(
                "Ya existe un cliente con RIF {}",
                rif
            )));
        }
    }

    let id = Uuid::new_v4();
    let now = Utc::now().into();
    let is_consumer_final = dto.rif.is_none();

    let client = clients::ActiveModel {
        id: Set(id),
        razon_social: Set(dto.name.clone()),
        nombre_comercial: Set(dto.trade_name.clone()),
        rif: Set(dto.rif.clone()),
        cedula: Set(None),
        domicilio_fiscal: Set(Some(dto.address.clone())),
        telefono: Set(dto.phone.clone()),
        email: Set(dto.email.clone()),
        es_consumidor_final: Set(is_consumer_final),
        es_contribuyente_especial: Set(dto.is_special_taxpayer.unwrap_or(false)),
        is_active: Set(true),
        created_by: Set(user_id),
        created_at: Set(now),
        updated_at: Set(now),
    };

    client.insert(db).await?;

    Ok(ClientResponse {
        id,
        rif: dto.rif,
        name: dto.name,
        trade_name: dto.trade_name,
        address: dto.address,
        phone: dto.phone,
        email: dto.email,
        is_special_taxpayer: dto.is_special_taxpayer.unwrap_or(false),
        invoice_count: Some(0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    })
}

/// Obtiene un cliente por ID.
pub async fn get_client(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<ClientResponse, AppError> {
    let client = clients::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Cliente con ID {} no encontrado", id)))?;

    // Count invoices
    let invoice_count = crate::entities::invoices::Entity::find()
        .filter(crate::entities::invoices::Column::ClientId.eq(id))
        .count(db)
        .await?;

    Ok(ClientResponse {
        id: client.id,
        rif: client.rif,
        name: client.razon_social,
        trade_name: client.nombre_comercial,
        address: client.domicilio_fiscal.unwrap_or_default(),
        phone: client.telefono,
        email: client.email,
        is_special_taxpayer: client.es_contribuyente_especial,
        invoice_count: Some(invoice_count),
        created_at: client.created_at.with_timezone(&Utc),
        updated_at: client.updated_at.with_timezone(&Utc),
    })
}

/// Lista clientes con filtros y paginación.
pub async fn list_clients(
    db: &DatabaseConnection,
    filters: ClientFilters,
) -> Result<PaginatedResponse<ClientListResponse>, AppError> {
    let page = filters.page.unwrap_or(1);
    let per_page = filters.per_page.unwrap_or(25);

    let mut query = clients::Entity::find()
        .filter(clients::Column::IsActive.eq(true))
        .order_by_desc(clients::Column::CreatedAt);

    if let Some(ref search) = filters.search {
        use sea_orm::Condition;
        query = query.filter(
            Condition::any()
                .add(clients::Column::RazonSocial.contains(search))
                .add(clients::Column::Rif.contains(search)),
        );
    }

    let total = query.clone().count(db).await?;
    let offset = (page.saturating_sub(1)) * per_page;

    let models = query.offset(Some(offset)).limit(Some(per_page)).all(db).await?;

    let data: Vec<ClientListResponse> = models
        .into_iter()
        .map(|c| ClientListResponse {
            id: c.id,
            rif: c.rif,
            name: c.razon_social,
            address: c.domicilio_fiscal.unwrap_or_default(),
            is_special_taxpayer: c.es_contribuyente_especial,
            created_at: c.created_at.with_timezone(&Utc),
        })
        .collect();

    Ok(PaginatedResponse::new(data, page, per_page, total))
}

/// Actualiza un cliente existente.
pub async fn update_client(
    db: &DatabaseConnection,
    id: Uuid,
    dto: UpdateClientRequest,
    _user_id: Uuid,
) -> Result<ClientResponse, AppError> {
    let client = clients::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Cliente con ID {} no encontrado", id)))?;

    let now = Utc::now().into();
    let mut active: clients::ActiveModel = client.into();

    if let Some(ref rif) = dto.rif {
        Rif::parse(rif).map_err(|e| {
            AppError::Validation(format!("RIF del cliente inválido: {}", e))
        })?;
        active.rif = Set(Some(rif.clone()));
    }
    if let Some(ref name) = dto.name {
        active.razon_social = Set(name.clone());
    }
    if let Some(ref trade_name) = dto.trade_name {
        active.nombre_comercial = Set(Some(trade_name.clone()));
    }
    if let Some(ref address) = dto.address {
        active.domicilio_fiscal = Set(Some(address.clone()));
    }
    if let Some(ref phone) = dto.phone {
        active.telefono = Set(Some(phone.clone()));
    }
    if let Some(ref email) = dto.email {
        active.email = Set(Some(email.clone()));
    }
    if let Some(is_special) = dto.is_special_taxpayer {
        active.es_contribuyente_especial = Set(is_special);
    }
    active.updated_at = Set(now);

    active.update(db).await?;

    get_client(db, id).await
}
