//! SeaORM entity for the `sales_book_entries` table.
//! Libro de ventas — mandatory monthly book per SENIAT regulations.
//! Consumer final sales without RIF can be grouped as daily summaries (resumen diario).

use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sales_book_entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub company_profile_id: Uuid,
    /// Book period: "2026-03"
    pub period: String,
    pub entry_date: DateTimeWithTimeZone,
    // ---- Client info (snapshot) ----
    /// Null for consumer final daily summaries
    pub client_rif: Option<String>,
    pub client_name: String,
    // ---- Document info ----
    pub invoice_number: String,
    pub control_number: String,
    pub invoice_date: Date,
    // ---- Amounts ----
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub total_amount: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub base_imponible_exenta: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub base_imponible_reducida: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub base_imponible_general: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub base_imponible_lujo: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub iva_reducida: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub iva_general: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub iva_lujo: Decimal,
    /// True if this is a grouped daily summary for consumer final clients
    pub es_resumen_diario: bool,
    /// Reference to originating invoice (optional)
    pub invoice_id: Option<Uuid>,
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

impl Related<super::company_profiles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CompanyProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
