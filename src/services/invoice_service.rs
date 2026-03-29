//! Servicio de facturas: orquesta dominio e infraestructura.
//!
//! Implementa persistencia real con SeaORM y numeración atómica.

use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, Statement, TransactionTrait,
};
use uuid::Uuid;

use crate::domain::invoice::{InvoiceBuilder, PaymentCondition};
use crate::domain::tax::iva::IvaRate;
use crate::dto::PaginatedResponse;
use crate::dto::invoice_dto::{
    CreateInvoiceRequest, InvoiceFilters, InvoiceItemResponse, InvoiceListResponse,
    InvoiceResponse, VoidInvoiceRequest,
};
use crate::entities::{invoice_items, invoices};
use crate::errors::AppError;

/// Obtiene el próximo número de factura atómicamente usando SELECT FOR UPDATE.
async fn next_invoice_number_atomic<C: ConnectionTrait>(
    txn: &C,
    company_profile_id: Uuid,
) -> Result<String, AppError> {
    // Lock the sequence row
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"SELECT id, current_value, prefix FROM numbering_sequences
               WHERE company_profile_id = $1 AND sequence_type = 'INVOICE' AND is_active = true
               FOR UPDATE"#,
            [company_profile_id.into()],
        ))
        .await
        .map_err(AppError::Database)?;

    let row = rows
        .first()
        .ok_or_else(|| AppError::BadRequest("No hay secuencia de numeración activa para facturas. Ejecute el seeder.".into()))?;

    let id: Uuid = row
        .try_get("", "id")
        .map_err(AppError::Database)?;
    let current: i64 = row
        .try_get("", "current_value")
        .map_err(AppError::Database)?;
    let prefix: Option<String> = row
        .try_get("", "prefix")
        .map_err(AppError::Database)?;

    let next_val = current + 1;

    // Update the sequence
    txn.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"UPDATE numbering_sequences SET current_value = $1, updated_at = NOW() WHERE id = $2"#,
        [next_val.into(), id.into()],
    ))
    .await
    .map_err(AppError::Database)?;

    let number = format!(
        "{}{:08}",
        prefix.unwrap_or_default(),
        next_val
    );
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

    let id: Uuid = row
        .try_get("", "id")
        .map_err(AppError::Database)?;
    let current: i64 = row
        .try_get("", "current_value")
        .map_err(AppError::Database)?;
    let prefix: String = row
        .try_get("", "prefix")
        .map_err(AppError::Database)?;
    let range_to: i64 = row
        .try_get("", "range_to")
        .map_err(AppError::Database)?;

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

/// Convierte el string de alícuota de IVA al enum del dominio.
pub fn parse_tax_rate(rate: &str) -> Result<IvaRate, AppError> {
    match rate.to_lowercase().as_str() {
        "general" => Ok(IvaRate::General),
        "reduced" => Ok(IvaRate::Reduced),
        "luxury" => Ok(IvaRate::Luxury),
        "exempt" => Ok(IvaRate::Exempt),
        _ => Err(AppError::BadRequest(format!(
            "Alícuota de IVA inválida: '{}'. Valores válidos: general, reduced, luxury, exempt",
            rate
        ))),
    }
}

/// Convierte el enum IvaRate al string para la response.
pub fn tax_rate_to_string(rate: IvaRate) -> String {
    match rate {
        IvaRate::General => "general".to_string(),
        IvaRate::Reduced => "reduced".to_string(),
        IvaRate::Luxury => "luxury".to_string(),
        IvaRate::Exempt => "exempt".to_string(),
    }
}

fn iva_rate_to_db(rate: IvaRate) -> (Decimal, String) {
    match rate {
        IvaRate::General => (Decimal::new(1600, 2), "GENERAL".to_string()),
        IvaRate::Reduced => (Decimal::new(800, 2), "REDUCIDA".to_string()),
        IvaRate::Luxury => (Decimal::new(3100, 2), "LUJO".to_string()),
        IvaRate::Exempt => (Decimal::ZERO, "EXENTO".to_string()),
    }
}

/// Crea una nueva factura con persistencia real y numeración atómica.
pub async fn create_invoice(
    db: &DatabaseConnection,
    dto: CreateInvoiceRequest,
    user_id: Uuid,
    company_profile_id: Uuid,
) -> Result<InvoiceResponse, AppError> {
    // Parsear condición de pago
    let payment_condition = match dto.payment_condition.to_lowercase().as_str() {
        "cash" => PaymentCondition::Cash,
        "credit" => PaymentCondition::Credit {
            days: dto.credit_days,
        },
        _ => {
            return Err(AppError::BadRequest(format!(
                "Condición de pago inválida: '{}'. Valores válidos: cash, credit",
                dto.payment_condition
            )));
        }
    };

    let client_address = dto.client_address.unwrap_or_default();

    // Todo dentro de una transacción para atomicidad
    let txn = db.begin().await?;

    // Obtener números atómicamente
    let invoice_number = next_invoice_number_atomic(&txn, company_profile_id).await?;
    let control_number = next_control_number_atomic(&txn, company_profile_id).await?;

    // Construir invoice data con el domain builder
    let mut builder = InvoiceBuilder::new()
        .invoice_number(&invoice_number)
        .control_number(&control_number)
        .client(
            dto.client_rif.as_deref(),
            &dto.client_name,
            &client_address,
        )
        .payment_condition(payment_condition)
        .currency(&dto.currency, dto.exchange_rate);

    if let Some(date) = dto.invoice_date {
        builder = builder.invoice_date(date);
    }

    for item in &dto.items {
        let tax_rate = parse_tax_rate(&item.tax_rate)?;
        builder = builder.add_item(&item.description, item.quantity, item.unit_price, tax_rate);
    }

    let invoice_data = builder.build().map_err(|errors| {
        let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        AppError::Validation(messages.join("; "))
    })?;

    let totals = invoice_data.calculate_totals();
    let now = Utc::now().into();
    let id = Uuid::new_v4();

    let (payment_condition_str, credit_days_val) = match &invoice_data.payment_condition {
        PaymentCondition::Cash => ("Contado".to_string(), None),
        PaymentCondition::Credit { days } => ("Crédito".to_string(), days.map(|d| d as i32)),
    };

    // Calculate per-rate subtotals from items
    let mut subtotal_exento = Decimal::ZERO;
    let mut subtotal_reducida = Decimal::ZERO;
    let mut subtotal_general = Decimal::ZERO;
    let mut subtotal_lujo = Decimal::ZERO;
    for item in &invoice_data.items {
        match item.tax_rate {
            IvaRate::Exempt => subtotal_exento += item.subtotal(),
            IvaRate::Reduced => subtotal_reducida += item.subtotal(),
            IvaRate::General => subtotal_general += item.subtotal(),
            IvaRate::Luxury => subtotal_lujo += item.subtotal(),
        }
    }

    // Resolver client_id: buscar por RIF o crear un cliente "consumidor final"
    let client_id = resolve_client_id(&txn, &dto.client_rif, &dto.client_name, &client_address, user_id).await?;

    // INSERT invoice
    let invoice_model = invoices::ActiveModel {
        id: Set(id),
        invoice_number: Set(invoice_data.invoice_number.clone()),
        control_number: Set(invoice_data.control_number.clone()),
        invoice_date: Set(now),
        due_date: Set(None),
        client_id: Set(client_id),
        company_profile_id: Set(company_profile_id),
        subtotal_exento: Set(subtotal_exento),
        subtotal_reducida: Set(subtotal_reducida),
        subtotal_general: Set(subtotal_general),
        subtotal_lujo: Set(subtotal_lujo),
        subtotal: Set(totals.subtotal),
        iva_reducida: Set(totals.tax_reduced),
        iva_general: Set(totals.tax_general),
        iva_lujo: Set(totals.tax_luxury),
        tax_amount: Set(totals.total_tax),
        total: Set(totals.grand_total),
        currency: Set(invoice_data.currency.clone()),
        exchange_rate_id: Set(None),
        exchange_rate_snapshot: Set(Some(invoice_data.exchange_rate)),
        status: Set("Emitida".to_string()),
        payment_condition: Set(payment_condition_str.clone()),
        credit_days: Set(credit_days_val),
        no_fiscal_credit: Set(invoice_data.no_fiscal_credit),
        notes: Set(None),
        annulment_reason: Set(None),
        created_by: Set(user_id),
        created_at: Set(now),
        updated_at: Set(now),
    };
    invoice_model.insert(&txn).await?;

    // INSERT invoice items
    let mut item_responses = Vec::new();
    for (idx, item) in invoice_data.items.iter().enumerate() {
        let (tax_rate_decimal, tax_type) = iva_rate_to_db(item.tax_rate);
        let item_id = Uuid::new_v4();
        let item_subtotal = item.subtotal();
        let item_tax = item.tax_amount();
        let item_total = item.total();

        let item_model = invoice_items::ActiveModel {
            id: Set(item_id),
            invoice_id: Set(id),
            line_number: Set((idx + 1) as i32),
            description: Set(item.description.clone()),
            quantity: Set(item.quantity),
            unit_price: Set(item.unit_price),
            discount: Set(Decimal::ZERO),
            subtotal: Set(item_subtotal),
            tax_rate: Set(tax_rate_decimal),
            tax_type: Set(tax_type),
            tax_amount: Set(item_tax),
            total: Set(item_total),
            product_code: Set(None),
            created_by: Set(user_id),
            created_at: Set(now),
            updated_at: Set(now),
        };
        item_model.insert(&txn).await?;

        item_responses.push(InvoiceItemResponse {
            description: item.description.clone(),
            quantity: item.quantity,
            unit_price: item.unit_price,
            tax_rate: tax_rate_to_string(item.tax_rate),
            subtotal: item_subtotal,
            tax_amount: item_tax,
            total: item_total,
        });
    }

    txn.commit().await?;

    let (pc_str, cd) = match &invoice_data.payment_condition {
        PaymentCondition::Cash => ("cash".to_string(), None),
        PaymentCondition::Credit { days } => ("credit".to_string(), *days),
    };

    Ok(InvoiceResponse {
        id,
        invoice_number: invoice_data.invoice_number,
        control_number: invoice_data.control_number,
        invoice_date: invoice_data.invoice_date,
        status: "issued".to_string(),
        client_rif: invoice_data.client_rif,
        client_name: invoice_data.client_name,
        client_address: invoice_data.client_address,
        items: item_responses,
        payment_condition: pc_str,
        credit_days: cd,
        no_fiscal_credit: invoice_data.no_fiscal_credit,
        currency: invoice_data.currency,
        exchange_rate: invoice_data.exchange_rate,
        subtotal: totals.subtotal,
        tax_general: totals.tax_general,
        tax_reduced: totals.tax_reduced,
        tax_luxury: totals.tax_luxury,
        total_tax: totals.total_tax,
        grand_total: totals.grand_total,
        created_by: user_id,
        created_at: Utc::now(),
    })
}

/// Resuelve el client_id: busca por RIF si existe, o crea un cliente temporal.
async fn resolve_client_id<C: ConnectionTrait>(
    txn: &C,
    client_rif: &Option<String>,
    client_name: &str,
    client_address: &str,
    user_id: Uuid,
) -> Result<Uuid, AppError> {
    use crate::entities::clients;

    if let Some(rif) = client_rif {
        // Validar formato del RIF antes de continuar
        crate::domain::numbering::rif::Rif::parse(rif).map_err(|e| {
            AppError::Validation(format!("RIF del cliente inválido: {}", e))
        })?;

        // Buscar cliente existente por RIF
        let existing = clients::Entity::find()
            .filter(clients::Column::Rif.eq(rif.as_str()))
            .one(txn)
            .await?;

        if let Some(client) = existing {
            return Ok(client.id);
        }

        // Crear cliente con RIF
        let id = Uuid::new_v4();
        let now = Utc::now().into();
        let client = clients::ActiveModel {
            id: Set(id),
            razon_social: Set(client_name.to_string()),
            nombre_comercial: Set(None),
            rif: Set(Some(rif.clone())),
            cedula: Set(None),
            domicilio_fiscal: Set(Some(client_address.to_string())),
            telefono: Set(None),
            email: Set(None),
            es_consumidor_final: Set(false),
            es_contribuyente_especial: Set(false),
            is_active: Set(true),
            created_by: Set(user_id),
            created_at: Set(now),
            updated_at: Set(now),
        };
        client.insert(txn).await?;
        Ok(id)
    } else {
        // Consumidor final sin RIF — crear cliente temporal
        let id = Uuid::new_v4();
        let now = Utc::now().into();
        let client = clients::ActiveModel {
            id: Set(id),
            razon_social: Set(client_name.to_string()),
            nombre_comercial: Set(None),
            rif: Set(None),
            cedula: Set(None),
            domicilio_fiscal: Set(Some(client_address.to_string())),
            telefono: Set(None),
            email: Set(None),
            es_consumidor_final: Set(true),
            es_contribuyente_especial: Set(false),
            is_active: Set(true),
            created_by: Set(user_id),
            created_at: Set(now),
            updated_at: Set(now),
        };
        client.insert(txn).await?;
        Ok(id)
    }
}

/// Obtiene una factura por ID.
pub async fn get_invoice(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<InvoiceResponse, AppError> {
    let invoice = invoices::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Factura con ID {} no encontrada", id)))?;

    let items = invoice_items::Entity::find()
        .filter(invoice_items::Column::InvoiceId.eq(id))
        .order_by_asc(invoice_items::Column::LineNumber)
        .all(db)
        .await?;

    // Get client info
    let client = crate::entities::clients::Entity::find_by_id(invoice.client_id)
        .one(db)
        .await?;

    let (client_rif, client_name, client_address) = match client {
        Some(c) => (c.rif, c.razon_social, c.domicilio_fiscal.unwrap_or_default()),
        None => (None, "Desconocido".to_string(), String::new()),
    };

    let item_responses: Vec<InvoiceItemResponse> = items
        .iter()
        .map(|item| InvoiceItemResponse {
            description: item.description.clone(),
            quantity: item.quantity,
            unit_price: item.unit_price,
            tax_rate: item.tax_type.to_lowercase(),
            subtotal: item.subtotal,
            tax_amount: item.tax_amount,
            total: item.total,
        })
        .collect();

    let (pc, cd) = match invoice.payment_condition.as_str() {
        "Crédito" => ("credit".to_string(), invoice.credit_days.map(|d| d as u32)),
        _ => ("cash".to_string(), None),
    };

    Ok(InvoiceResponse {
        id: invoice.id,
        invoice_number: invoice.invoice_number,
        control_number: invoice.control_number,
        invoice_date: invoice.invoice_date.date_naive(),
        status: invoice.status.to_lowercase(),
        client_rif,
        client_name,
        client_address,
        items: item_responses,
        payment_condition: pc,
        credit_days: cd,
        no_fiscal_credit: invoice.no_fiscal_credit,
        currency: invoice.currency,
        exchange_rate: invoice.exchange_rate_snapshot.unwrap_or(Decimal::ONE),
        subtotal: invoice.subtotal,
        tax_general: invoice.iva_general,
        tax_reduced: invoice.iva_reducida,
        tax_luxury: invoice.iva_lujo,
        total_tax: invoice.tax_amount,
        grand_total: invoice.total,
        created_by: invoice.created_by,
        created_at: invoice.created_at.with_timezone(&Utc),
    })
}

/// Lista facturas con filtros y paginación.
pub async fn list_invoices(
    db: &DatabaseConnection,
    filters: InvoiceFilters,
) -> Result<PaginatedResponse<InvoiceListResponse>, AppError> {
    let page = filters.page.unwrap_or(1);
    let per_page = filters.per_page.unwrap_or(25);

    let mut query = invoices::Entity::find().order_by_desc(invoices::Column::CreatedAt);

    if let Some(status) = &filters.status {
        query = query.filter(invoices::Column::Status.eq(status.as_str()));
    }
    if let Some(from) = filters.from {
        use sea_orm::sea_query::Expr;
        query = query.filter(Expr::col(invoices::Column::InvoiceDate).gte(from));
    }
    if let Some(to) = filters.to {
        use sea_orm::sea_query::Expr;
        query = query.filter(Expr::col(invoices::Column::InvoiceDate).lte(to));
    }

    let total = query.clone().count(db).await?;
    let offset = (page.saturating_sub(1)) * per_page;

    let invoice_models = query
        .offset(Some(offset))
        .limit(Some(per_page))
        .all(db)
        .await?;

    let mut data = Vec::new();
    for inv in invoice_models {
        let client = crate::entities::clients::Entity::find_by_id(inv.client_id)
            .one(db)
            .await?;
        let (client_name, client_rif) = match client {
            Some(c) => (c.razon_social, c.rif),
            None => ("Desconocido".to_string(), None),
        };

        data.push(InvoiceListResponse {
            id: inv.id,
            invoice_number: inv.invoice_number,
            control_number: inv.control_number,
            invoice_date: inv.invoice_date.date_naive(),
            status: inv.status.to_lowercase(),
            client_name,
            client_rif,
            grand_total: inv.total,
            currency: inv.currency,
            created_at: inv.created_at.with_timezone(&Utc),
        });
    }

    Ok(PaginatedResponse::new(data, page, per_page, total))
}

/// Anula una factura (marca como Anulada, NUNCA elimina).
pub async fn void_invoice(
    db: &DatabaseConnection,
    id: Uuid,
    _user_id: Uuid,
    request: VoidInvoiceRequest,
) -> Result<InvoiceResponse, AppError> {
    let invoice = invoices::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Factura con ID {} no encontrada", id)))?;

    if invoice.status == "Anulada" {
        return Err(AppError::BadRequest(
            "La factura ya está anulada".to_string(),
        ));
    }

    let now = Utc::now().into();
    let mut active: invoices::ActiveModel = invoice.into();
    active.status = Set("Anulada".to_string());
    active.annulment_reason = Set(Some(request.reason));
    active.updated_at = Set(now);
    active.update(db).await?;

    // Return the updated invoice
    get_invoice(db, id).await
}
