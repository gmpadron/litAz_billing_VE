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
    /// Empresa a la que pertenece el evento (nullable para registros históricos)
    pub company_profile_id: Option<Uuid>,
    /// No updated_at — audit records are immutable
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company_profiles::Entity",
        from = "Column::CompanyProfileId",
        to = "super::company_profiles::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    CompanyProfile,
}

impl Related<super::company_profiles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CompanyProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
