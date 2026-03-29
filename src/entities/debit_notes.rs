//! SeaORM entity for the `debit_notes` table.
//! Notas de débito — issued to add charges to a previous invoice.

use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "debit_notes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub debit_note_number: String,
    /// SENIAT control number: 00-XXXXXXXX
    #[sea_orm(unique)]
    pub control_number: String,
    /// The invoice this note augments
    pub original_invoice_id: Uuid,
    pub client_id: Uuid,
    pub company_profile_id: Uuid,
    pub issue_date: DateTimeWithTimeZone,
    pub reason: String,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub subtotal: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub tax_amount: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub total: Decimal,
    pub currency: String,
    pub exchange_rate_id: Option<Uuid>,
    #[sea_orm(column_type = "Decimal(Some((18, 6)))", nullable)]
    pub exchange_rate_snapshot: Option<Decimal>,
    /// Emitida | Anulada
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::invoices::Entity",
        from = "Column::OriginalInvoiceId",
        to = "super::invoices::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    OriginalInvoice,
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
    #[sea_orm(
        belongs_to = "super::exchange_rates::Entity",
        from = "Column::ExchangeRateId",
        to = "super::exchange_rates::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    ExchangeRate,
}

impl Related<super::invoices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OriginalInvoice.def()
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

impl Related<super::exchange_rates::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExchangeRate.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
