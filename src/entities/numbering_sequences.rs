//! SeaORM entity for the `numbering_sequences` table.
//! Controls invoice and document numbering. Use SELECT FOR UPDATE for atomicity.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "numbering_sequences")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Type: INVOICE, CREDIT_NOTE, DEBIT_NOTE, DELIVERY_NOTE
    pub sequence_type: String,
    /// Optional prefix e.g. "FAC-", "NC-", "ND-"
    pub prefix: Option<String>,
    /// Last used value — next = current_value + 1
    pub current_value: i64,
    pub min_value: i64,
    pub max_value: Option<i64>,
    pub is_active: bool,
    pub company_profile_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
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
