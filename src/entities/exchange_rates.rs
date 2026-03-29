//! SeaORM entity for the `exchange_rates` table.
//! Stores daily BCV exchange rates (Bs/USD).

use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "exchange_rates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// The date this rate applies to
    pub rate_date: Date,
    /// Base currency (VES)
    pub base_currency: String,
    /// Quote currency (USD)
    pub quote_currency: String,
    /// How many Bs per 1 USD — DECIMAL(18,6)
    #[sea_orm(column_type = "Decimal(Some((18, 6)))")]
    pub rate: Decimal,
    /// Source: BCV, PARALLEL, MANUAL
    pub source: String,
    pub created_by: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::invoices::Entity")]
    Invoices,
    #[sea_orm(has_many = "super::credit_notes::Entity")]
    CreditNotes,
    #[sea_orm(has_many = "super::debit_notes::Entity")]
    DebitNotes,
}

impl Related<super::invoices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Invoices.def()
    }
}

impl Related<super::credit_notes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreditNotes.def()
    }
}

impl Related<super::debit_notes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DebitNotes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
