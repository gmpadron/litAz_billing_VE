//! Servicio de notas de débito: persistencia real con SeaORM.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, Statement, TransactionTrait,
};
use uuid::Uuid;

use crate::domain::invoice::debit_note::DebitNoteBuilder;
use crate::dto::PaginatedResponse;
use crate::dto::debit_note_dto::{
    CreateDebitNoteRequest, DebitNoteFilters, DebitNoteItemResponse, DebitNoteListResponse,
    DebitNoteResponse,
};
use crate::entities::{debit_notes, invoices};
use crate::errors::AppError;
use crate::services::invoice_service::tax_rate_to_string;

/// Obtiene el próximo número de nota de débito atómicamente usando SELECT FOR UPDATE.
async fn next_debit_note_number_atomic<C: ConnectionTrait>(
    txn: &C,
    company_profile_id: Uuid,
) -> Result<String, AppError> {
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"SELECT id, current_value, prefix FROM numbering_sequences
               WHERE company_profile_id = $1 AND sequence_type = 'DEBIT_NOTE' AND is_active = true
               FOR UPDATE"#,
            [company_profile_id.into()],
        ))
        .await
        .map_err(AppError::Database)?;

    let row = rows.first().ok_or_else(|| {
        AppError::BadRequest(
            "No hay secuencia de numeración activa para notas de débito. Ejecute el seeder.".into(),
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

/// Crea una nueva nota de débito con persistencia real.
pub async fn create_debit_note(
    db: &DatabaseConnection,
    dto: CreateDebitNoteRequest,
    user_id: Uuid,
    company_profile_id: Uuid,
) -> Result<DebitNoteResponse, AppError> {
    // Buscar factura original
    let original_invoice = invoices::Entity::find()
        .filter(invoices::Column::InvoiceNumber.eq(dto.original_invoice_number.as_str()))
        .one(db)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Factura original {} no encontrada",
                dto.original_invoice_number
            ))
        })?;

    let mut builder = DebitNoteBuilder::new()
        .original_invoice(&dto.original_invoice_number)
        .reason(&dto.reason);

    if let Some(date) = dto.issue_date {
        builder = builder.issue_date(date);
    }

    for item in &dto.items {
        let tax_rate = crate::services::invoice_service::parse_tax_rate(&item.tax_rate)?;
        builder = builder.add_item(&item.description, item.quantity, item.unit_price, tax_rate);
    }

    let mut note_data = builder.build().map_err(|errors| {
        let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        AppError::Validation(messages.join("; "))
    })?;

    let subtotal = note_data.subtotal();
    let total_tax = note_data.total_tax();
    let grand_total = note_data.grand_total();

    let txn = db.begin().await?;

    // Asignar número de nota de débito y número de control atómicamente
    let debit_note_number = next_debit_note_number_atomic(&txn, company_profile_id).await?;
    let control_number = next_control_number_atomic(&txn, company_profile_id).await?;

    note_data.debit_note_number = debit_note_number;

    let id = Uuid::new_v4();
    let now = Utc::now().into();

    let debit_note = debit_notes::ActiveModel {
        id: Set(id),
        debit_note_number: Set(note_data.debit_note_number.clone()),
        control_number: Set(control_number),
        original_invoice_id: Set(original_invoice.id),
        client_id: Set(original_invoice.client_id),
        company_profile_id: Set(company_profile_id),
        issue_date: Set(now),
        reason: Set(note_data.reason.clone()),
        subtotal: Set(subtotal),
        tax_amount: Set(total_tax),
        total: Set(grand_total),
        currency: Set(original_invoice.currency.clone()),
        exchange_rate_id: Set(None),
        exchange_rate_snapshot: Set(original_invoice.exchange_rate_snapshot),
        status: Set("Emitida".to_string()),
        notes: Set(None),
        created_by: Set(user_id),
        created_at: Set(now),
        updated_at: Set(now),
    };

    debit_note.insert(&txn).await?;
    txn.commit().await?;

    let item_responses: Vec<DebitNoteItemResponse> = note_data
        .items
        .iter()
        .map(|item| DebitNoteItemResponse {
            description: item.description.clone(),
            quantity: item.quantity,
            unit_price: item.unit_price,
            tax_rate: tax_rate_to_string(item.tax_rate),
            subtotal: item.subtotal(),
            tax_amount: item.tax_amount(),
            total: item.total(),
        })
        .collect();

    Ok(DebitNoteResponse {
        id,
        debit_note_number: note_data.debit_note_number,
        original_invoice_number: note_data.original_invoice_number,
        issue_date: note_data.issue_date,
        status: "issued".to_string(),
        reason: note_data.reason,
        client_rif: note_data.client_rif,
        client_name: note_data.client_name,
        items: item_responses,
        subtotal,
        total_tax,
        grand_total,
        created_by: user_id,
        created_at: Utc::now(),
    })
}

/// Obtiene una nota de débito por ID.
pub async fn get_debit_note(
    db: &DatabaseConnection,
    id: Uuid,
    company_profile_id: Uuid,
) -> Result<DebitNoteResponse, AppError> {
    let note = debit_notes::Entity::find_by_id(id)
        .filter(debit_notes::Column::CompanyProfileId.eq(company_profile_id))
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Nota de débito con ID {} no encontrada", id)))?;

    let original = invoices::Entity::find_by_id(note.original_invoice_id)
        .one(db)
        .await?;
    let original_number = original.map(|i| i.invoice_number).unwrap_or_default();

    let client = crate::entities::clients::Entity::find_by_id(note.client_id)
        .one(db)
        .await?;
    let (client_rif, client_name) = match client {
        Some(c) => (c.rif, c.razon_social),
        None => (None, "Desconocido".to_string()),
    };

    Ok(DebitNoteResponse {
        id: note.id,
        debit_note_number: note.debit_note_number,
        original_invoice_number: original_number,
        issue_date: note.issue_date.date_naive(),
        status: note.status.to_lowercase(),
        reason: note.reason,
        client_rif,
        client_name,
        items: vec![],
        subtotal: note.subtotal,
        total_tax: note.tax_amount,
        grand_total: note.total,
        created_by: note.created_by,
        created_at: note.created_at.with_timezone(&Utc),
    })
}

/// Lista notas de débito con filtros y paginación.
pub async fn list_debit_notes(
    db: &DatabaseConnection,
    filters: DebitNoteFilters,
    company_profile_id: Uuid,
) -> Result<PaginatedResponse<DebitNoteListResponse>, AppError> {
    let page = filters.page.unwrap_or(1);
    let per_page = filters.per_page.unwrap_or(25);

    let mut query = debit_notes::Entity::find()
        .filter(debit_notes::Column::CompanyProfileId.eq(company_profile_id))
        .order_by_desc(debit_notes::Column::CreatedAt);

    if let Some(from) = filters.from {
        use sea_orm::sea_query::Expr;
        query = query.filter(Expr::col(debit_notes::Column::IssueDate).gte(from));
    }
    if let Some(to) = filters.to {
        use sea_orm::sea_query::Expr;
        query = query.filter(Expr::col(debit_notes::Column::IssueDate).lte(to));
    }

    let total = query.clone().count(db).await?;
    let offset = (page.saturating_sub(1)) * per_page;

    let models = query.offset(Some(offset)).limit(Some(per_page)).all(db).await?;

    let mut data = Vec::new();
    for note in models {
        let original = invoices::Entity::find_by_id(note.original_invoice_id)
            .one(db)
            .await?;
        let original_number = original.map(|i| i.invoice_number).unwrap_or_default();

        data.push(DebitNoteListResponse {
            id: note.id,
            debit_note_number: note.debit_note_number,
            original_invoice_number: original_number,
            issue_date: note.issue_date.date_naive(),
            status: note.status.to_lowercase(),
            reason: note.reason,
            grand_total: note.total,
            created_at: note.created_at.with_timezone(&Utc),
        });
    }

    Ok(PaginatedResponse::new(data, page, per_page, total))
}
