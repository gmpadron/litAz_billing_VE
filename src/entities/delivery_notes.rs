//! SeaORM entity for the `delivery_notes` table.
//! Órdenes de entrega / guías de despacho.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "delivery_notes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub delivery_note_number: String,
    /// SENIAT control number: 00-XXXXXXXX
    #[sea_orm(unique)]
    pub control_number: String,
    /// Associated invoice (optional — delivery can precede invoice)
    pub invoice_id: Option<Uuid>,
    pub client_id: Uuid,
    pub company_profile_id: Uuid,
    pub issue_date: DateTimeWithTimeZone,
    /// Delivery address (can differ from fiscal address)
    pub delivery_address: Option<String>,
    pub notes: Option<String>,
    /// Emitida | Entregada — nunca se anula, inmutable una vez emitida
    pub status: String,
    pub created_by: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::invoices::Entity",
        from = "Column::InvoiceId",
        to = "super::invoices::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Invoice,
    #[sea_orm(
        belongs_to = "super::clients::Entity",
        from = "Column::ClientId",
        to = "super::clients::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    Client,
    #[sea_orm(
        belongs_to = "super::company_profiles::Entity",
        from = "Column::CompanyProfileId",
        to = "super::company_profiles::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    CompanyProfile,
}

impl Related<super::invoices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Invoice.def()
    }
}

impl Related<super::clients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Client.def()
    }
}

impl Related<super::company_profiles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CompanyProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
