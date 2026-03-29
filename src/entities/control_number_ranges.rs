//! SeaORM entity for the `control_number_ranges` table.
//! Manages control number ranges assigned by authorized printers (imprentas autorizadas).

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "control_number_ranges")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub company_profile_id: Uuid,
    /// Prefix portion of the control number e.g. "00"
    pub prefix: String,
    /// Start of the range (numeric portion)
    pub range_from: i64,
    /// End of the range (inclusive)
    pub range_to: i64,
    /// Current pointer — last issued value
    pub current_value: i64,
    /// Only one range should be active per company at a time
    pub is_active: bool,
    /// Name/reference of the authorized printer
    pub imprenta_autorizada: Option<String>,
    pub fecha_autorizacion: Option<Date>,
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
