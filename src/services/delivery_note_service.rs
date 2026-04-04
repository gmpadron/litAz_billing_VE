//! Servicio de guías de despacho: persistencia real con SeaORM.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, Statement, TransactionTrait,
};
use uuid::Uuid;

use crate::domain::invoice::delivery_note::DeliveryNoteBuilder;
use crate::domain::numbering::rif::Rif;
use crate::dto::PaginatedResponse;
use crate::dto::delivery_note_dto::{
    CreateDeliveryNoteRequest, DeliveryNoteFilters, DeliveryNoteItemResponse,
    DeliveryNoteListResponse, DeliveryNoteResponse,
};
use crate::entities::{delivery_notes, invoices};
use crate::errors::AppError;

/// Obtiene el próximo número de control atómicamente usando SELECT FOR UPDATE.
async fn next_control_number_atomic<C: ConnectionTrait>(
    txn: &C,
    company_profile_id: Uuid,
) -> Result<String, AppError> {
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"SELECT id, current_value, prefix, range_to FROM control_number_ranges
               WHERE company_profile_id = $1 AND is_active = true
               FOR UPDATE"#,
            [company_profile_id.into()],
        ))
        .await
        .map_err(AppError::Database)?;

    let row = rows.first().ok_or_else(|| {
        AppError::BadRequest(
            "No hay rango de números de control activo. Ejecute el seeder.".into(),
        )
    })?;

    let id: Uuid = row.try_get("", "id").map_err(AppError::Database)?;
    let current: i64 = row.try_get("", "current_value").map_err(AppError::Database)?;
    let prefix: String = row.try_get("", "prefix").map_err(AppError::Database)?;
    let range_to: i64 = row.try_get("", "range_to").map_err(AppError::Database)?;

    let next_val = current + 1;

    if next_val > range_to {
        return Err(AppError::BadRequest(
            "Rango de números de control agotado. Solicite un nuevo rango a la imprenta autorizada."
                .into(),
        ));
    }

    txn.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"UPDATE control_number_ranges SET current_value = $1, updated_at = NOW() WHERE id = $2"#,
        [next_val.into(), id.into()],
    ))
    .await
    .map_err(AppError::Database)?;

    Ok(format!("{}-{:08}", prefix, next_val))
}

/// Obtiene el próximo número de guía de despacho atómicamente usando SELECT FOR UPDATE.
async fn next_delivery_note_number_atomic<C: ConnectionTrait>(
    txn: &C,
    company_profile_id: Uuid,
) -> Result<String, AppError> {
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"SELECT id, current_value, prefix FROM numbering_sequences
               WHERE company_profile_id = $1 AND sequence_type = 'DELIVERY_NOTE' AND is_active = true
               FOR UPDATE"#,
            [company_profile_id.into()],
        ))
        .await
        .map_err(AppError::Database)?;

    let row = rows.first().ok_or_else(|| {
        AppError::BadRequest(
            "No hay secuencia de numeración activa para guías de despacho. Ejecute el seeder."
                .into(),
        )
    })?;

    let id: Uuid = row.try_get("", "id").map_err(AppError::Database)?;
    let current: i64 = row.try_get("", "current_value").map_err(AppError::Database)?;
    let prefix: Option<String> = row.try_get("", "prefix").map_err(AppError::Database)?;

    let next_val = current + 1;

    txn.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"UPDATE numbering_sequences SET current_value = $1, updated_at = NOW() WHERE id = $2"#,
        [next_val.into(), id.into()],
    ))
    .await
    .map_err(AppError::Database)?;

    let number = format!("{}{:08}", prefix.unwrap_or_default(), next_val);
    Ok(number)
}

/// Crea una nueva guía de despacho con persistencia real.
pub async fn create_delivery_note(
    db: &DatabaseConnection,
    dto: CreateDeliveryNoteRequest,
    user_id: Uuid,
    company_profile_id: Uuid,
) -> Result<DeliveryNoteResponse, AppError> {
    let mut builder = DeliveryNoteBuilder::new().recipient(
            &dto.recipient_name,
            dto.recipient_rif.as_deref(),
            &dto.destination_address,
        );

    if let Some(inv) = &dto.related_invoice_number {
        builder = builder.related_invoice(inv.as_str());
    }

    if let Some(date) = dto.issue_date {
        builder = builder.issue_date(date);
    }

    if let (Some(vehicle), Some(driver)) = (&dto.vehicle_info, &dto.driver_name) {
        builder = builder.vehicle(vehicle.as_str(), driver.as_str());
    }

    for item in &dto.items {
        builder = builder.add_item(&item.description, item.quantity, &item.unit);
    }

    let mut note_data = builder.build().map_err(|errors| {
        let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        AppError::Validation(messages.join("; "))
    })?;

    // Resolver invoice_id y client_id si hay factura relacionada
    let (invoice_id, client_id) = if let Some(ref inv_num) = dto.related_invoice_number {
        let inv = invoices::Entity::find()
            .filter(invoices::Column::InvoiceNumber.eq(inv_num.as_str()))
            .one(db)
            .await?;
        match inv {
            Some(i) => (Some(i.id), i.client_id),
            None => {
                // Crear un client stub
                let cid = create_client_stub(db, &dto.recipient_name, dto.recipient_rif.as_deref(), user_id, company_profile_id).await?;
                (None, cid)
            }
        }
    } else {
        let cid = create_client_stub(db, &dto.recipient_name, dto.recipient_rif.as_deref(), user_id, company_profile_id).await?;
        (None, cid)
    };

    let txn = db.begin().await?;

    // Asignar número de guía de despacho y número de control atómicamente
    let delivery_note_number = next_delivery_note_number_atomic(&txn, company_profile_id).await?;
    let control_number = next_control_number_atomic(&txn, company_profile_id).await?;
    note_data.delivery_note_number = delivery_note_number;

    let id = Uuid::new_v4();
    let now = Utc::now().into();

    let delivery = delivery_notes::ActiveModel {
        id: Set(id),
        delivery_note_number: Set(note_data.delivery_note_number.clone()),
        control_number: Set(control_number),
        invoice_id: Set(invoice_id),
        client_id: Set(client_id),
        company_profile_id: Set(company_profile_id),
        issue_date: Set(now),
        delivery_address: Set(Some(note_data.destination_address.clone())),
        notes: Set(None),
        status: Set("Emitida".to_string()),
        created_by: Set(user_id),
        created_at: Set(now),
        updated_at: Set(now),
    };

    delivery.insert(&txn).await?;
    txn.commit().await?;

    let item_responses: Vec<DeliveryNoteItemResponse> = note_data
        .items
        .iter()
        .map(|item| DeliveryNoteItemResponse {
            description: item.description.clone(),
            quantity: item.quantity,
            unit: item.unit.clone(),
        })
        .collect();

    Ok(DeliveryNoteResponse {
        id,
        delivery_note_number: note_data.delivery_note_number,
        related_invoice_number: note_data.related_invoice_number,
        issue_date: note_data.issue_date,
        status: "issued".to_string(),
        recipient_name: note_data.recipient_name,
        recipient_rif: note_data.recipient_rif,
        destination_address: note_data.destination_address,
        vehicle_info: note_data.vehicle_info,
        driver_name: note_data.driver_name,
        items: item_responses,
        created_by: user_id,
        created_at: Utc::now(),
    })
}

async fn create_client_stub(
    db: &sea_orm::DatabaseConnection,
    name: &str,
    rif: Option<&str>,
    user_id: Uuid,
    company_profile_id: Uuid,
) -> Result<Uuid, AppError> {
    use crate::entities::clients;

    // Validate RIF format if provided
    if let Some(rif_val) = rif {
        Rif::parse(rif_val).map_err(|e| {
            AppError::Validation(format!("RIF inválido en guía de despacho: {}", e))
        })?;
    }

    // Try to find existing by RIF within the same company
    if let Some(rif_val) = rif {
        let existing = clients::Entity::find()
            .filter(clients::Column::Rif.eq(rif_val))
            .filter(clients::Column::CompanyProfileId.eq(company_profile_id))
            .one(db)
            .await?;
        if let Some(c) = existing {
            return Ok(c.id);
        }
    }

    let id = Uuid::new_v4();
    let now = Utc::now().into();
    let client = clients::ActiveModel {
        id: Set(id),
        razon_social: Set(name.to_string()),
        nombre_comercial: Set(None),
        rif: Set(rif.map(|s| s.to_string())),
        cedula: Set(None),
        domicilio_fiscal: Set(None),
        telefono: Set(None),
        email: Set(None),
        es_consumidor_final: Set(rif.is_none()),
        es_contribuyente_especial: Set(false),
        is_active: Set(true),
        created_by: Set(user_id),
        created_at: Set(now),
        updated_at: Set(now),
        company_profile_id: Set(company_profile_id),
    };
    client.insert(db).await?;
    Ok(id)
}

/// Obtiene una guía de despacho por ID.
pub async fn get_delivery_note(
    db: &DatabaseConnection,
    id: Uuid,
    company_profile_id: Uuid,
) -> Result<DeliveryNoteResponse, AppError> {
    let note = delivery_notes::Entity::find_by_id(id)
        .filter(delivery_notes::Column::CompanyProfileId.eq(company_profile_id))
        .one(db)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Guía de despacho con ID {} no encontrada", id))
        })?;

    // Get related invoice number
    let related_invoice = if let Some(inv_id) = note.invoice_id {
        invoices::Entity::find_by_id(inv_id)
            .one(db)
            .await?
            .map(|i| i.invoice_number)
    } else {
        None
    };

    let client = crate::entities::clients::Entity::find_by_id(note.client_id)
        .one(db)
        .await?;
    let (recipient_name, recipient_rif) = match client {
        Some(c) => (c.razon_social, c.rif),
        None => ("Desconocido".to_string(), None),
    };

    Ok(DeliveryNoteResponse {
        id: note.id,
        delivery_note_number: note.delivery_note_number,
        related_invoice_number: related_invoice,
        issue_date: note.issue_date.date_naive(),
        status: note.status.to_lowercase(),
        recipient_name,
        recipient_rif,
        destination_address: note.delivery_address.unwrap_or_default(),
        vehicle_info: None,
        driver_name: None,
        items: vec![],
        created_by: note.created_by,
        created_at: note.created_at.with_timezone(&Utc),
    })
}

/// Lista guías de despacho con filtros y paginación.
pub async fn list_delivery_notes(
    db: &DatabaseConnection,
    filters: DeliveryNoteFilters,
    company_profile_id: Uuid,
) -> Result<PaginatedResponse<DeliveryNoteListResponse>, AppError> {
    let page = filters.page.unwrap_or(1);
    let per_page = filters.per_page.unwrap_or(25);

    let mut query = delivery_notes::Entity::find()
        .filter(delivery_notes::Column::CompanyProfileId.eq(company_profile_id))
        .order_by_desc(delivery_notes::Column::CreatedAt);

    if let Some(from) = filters.from {
        use sea_orm::sea_query::Expr;
        query = query.filter(Expr::col(delivery_notes::Column::IssueDate).gte(from));
    }
    if let Some(to) = filters.to {
        use sea_orm::sea_query::Expr;
        query = query.filter(Expr::col(delivery_notes::Column::IssueDate).lte(to));
    }

    let total = query.clone().count(db).await?;
    let offset = (page.saturating_sub(1)) * per_page;

    let models = query.offset(Some(offset)).limit(Some(per_page)).all(db).await?;

    let mut data = Vec::new();
    for note in models {
        let related = if let Some(inv_id) = note.invoice_id {
            invoices::Entity::find_by_id(inv_id)
                .one(db)
                .await?
                .map(|i| i.invoice_number)
        } else {
            None
        };

        let client = crate::entities::clients::Entity::find_by_id(note.client_id)
            .one(db)
            .await?;
        let recipient_name = client.map(|c| c.razon_social).unwrap_or_default();

        data.push(DeliveryNoteListResponse {
            id: note.id,
            delivery_note_number: note.delivery_note_number,
            related_invoice_number: related,
            issue_date: note.issue_date.date_naive(),
            status: note.status.to_lowercase(),
            recipient_name,
            created_at: note.created_at.with_timezone(&Utc),
        });
    }

    Ok(PaginatedResponse::new(data, page, per_page, total))
}
