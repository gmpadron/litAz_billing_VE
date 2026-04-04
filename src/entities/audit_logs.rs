//! SeaORM entity for the `audit_logs` table.
//! Immutable audit trail — never update, never delete.
//! Minimum retention: 10 years per SENIAT requirements.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "audit_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// UUID from JWT — who performed the action
    pub user_id: Uuid,
    /// Action: CREATE | UPDATE | VIEW | EXPORT | DECLARE
    pub action: String,
    /// Type of entity: invoice, credit_note, debit_note, etc.
    pub entity_type: String,
    /// UUID of the affected entity
    pub entity_id: Uuid,
    /// JSONB diff of what changed (before/after snapshot)
    pub changes: Option<Json>,
    /// Additional context: IP address, user agent, etc.
    pub metadata: Option<Json>,
    /// No updated_at — audit records are immutable
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
