//! Servicio de auditoría: registra operaciones en `audit_logs`.
//!
//! El SENIAT exige conservar registros de todas las operaciones fiscales
//! por un mínimo de 10 años (Código Orgánico Tributario).
//! Los registros de auditoría son inmutables — nunca se modifican ni eliminan.

use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

use crate::entities::audit_logs;
use crate::errors::AppError;

/// Tipos de acción registrables (sincronizados con `chk_audit_action`).
pub enum AuditAction {
    Create,
    Update,
    View,
    Export,
    Declare,
}

impl AuditAction {
    fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Create => "CREATE",
            AuditAction::Update => "UPDATE",
            AuditAction::View => "VIEW",
            AuditAction::Export => "EXPORT",
            AuditAction::Declare => "DECLARE",
        }
    }
}

/// Tipos de entidad registrables.
pub enum AuditEntity {
    Invoice,
    CreditNote,
    DebitNote,
    DeliveryNote,
    WithholdingIva,
    WithholdingIslr,
    Client,
    Company,
}

impl AuditEntity {
    fn as_str(&self) -> &'static str {
        match self {
            AuditEntity::Invoice => "invoice",
            AuditEntity::CreditNote => "credit_note",
            AuditEntity::DebitNote => "debit_note",
            AuditEntity::DeliveryNote => "delivery_note",
            AuditEntity::WithholdingIva => "withholding_iva",
            AuditEntity::WithholdingIslr => "withholding_islr",
            AuditEntity::Client => "client",
            AuditEntity::Company => "company",
        }
    }
}

/// Registra una operación en audit_logs.
///
/// Los errores de escritura en audit_logs se loguean pero NO propagan —
/// el fallo de auditoría no debe bloquear la operación principal.
pub async fn log(
    db: &DatabaseConnection,
    user_id: Uuid,
    action: AuditAction,
    entity: AuditEntity,
    entity_id: Uuid,
    metadata: Option<serde_json::Value>,
) {
    let result = write_log(db, user_id, action, entity, entity_id, metadata).await;
    if let Err(e) = result {
        log::error!("Error escribiendo en audit_logs: {}", e);
    }
}

async fn write_log(
    db: &DatabaseConnection,
    user_id: Uuid,
    action: AuditAction,
    entity: AuditEntity,
    entity_id: Uuid,
    metadata: Option<serde_json::Value>,
) -> Result<(), AppError> {
    let log_entry = audit_logs::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        action: Set(action.as_str().to_string()),
        entity_type: Set(entity.as_str().to_string()),
        entity_id: Set(entity_id),
        changes: Set(None),
        metadata: Set(metadata),
        created_at: Set(Utc::now().into()),
    };
    log_entry.insert(db).await?;
    Ok(())
}
