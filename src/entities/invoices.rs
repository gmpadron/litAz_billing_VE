//! SeaORM entity for the `invoices` table.
//! Invoices are IMMUTABLE once issued — never delete, use Anulada status instead.

use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "invoices")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Internal sequential invoice number (unique, no gaps)
    #[sea_orm(unique)]
    pub invoice_number: String,
    /// SENIAT control number format: 00-XXXXXXXX (unique, assigned by imprenta)
    #[sea_orm(unique)]
    pub control_number: String,
    pub invoice_date: DateTimeWithTimeZone,
    pub due_date: Option<DateTimeWithTimeZone>,
    pub client_id: Uuid,
    pub company_profile_id: Uuid,
    // ---- Tax-base breakdown by IVA rate ----
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub subtotal_exento: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub subtotal_reducida: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub subtotal_general: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub subtotal_lujo: Decimal,
    /// Sum of all subtotals
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub subtotal: Decimal,
    // ---- IVA amounts by rate ----
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub iva_reducida: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub iva_general: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub iva_lujo: Decimal,
    /// Total IVA = iva_reducida + iva_general + iva_lujo
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub tax_amount: Decimal,
    /// Grand total = subtotal + tax_amount
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub total: Decimal,
    // ---- Currency ----
    /// ISO currency code: VES or USD
    pub currency: String,
    pub exchange_rate_id: Option<Uuid>,
    /// Snapshot of exchange rate at time of invoice (immutable reference)
    #[sea_orm(column_type = "Decimal(Some((18, 6)))", nullable)]
    pub exchange_rate_snapshot: Option<Decimal>,
    // ---- Status and payment ----
    /// Emitida | Anulada — never delete rows
    pub status: String,
    /// Contado | Crédito
    pub payment_condition: String,
    /// Days of credit (required when payment_condition = Crédito)
    pub credit_days: Option<i32>,
    /// "SIN DERECHO A CRÉDITO FISCAL" — required for consumer final invoices
    pub no_fiscal_credit: bool,
    pub notes: Option<String>,
    pub annulment_reason: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
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
    #[sea_orm(has_many = "super::invoice_items::Entity")]
    InvoiceItems,
    #[sea_orm(has_many = "super::credit_notes::Entity")]
    CreditNotes,
    #[sea_orm(has_many = "super::debit_notes::Entity")]
    DebitNotes,
    #[sea_orm(has_many = "super::delivery_notes::Entity")]
    DeliveryNotes,
    #[sea_orm(has_many = "super::tax_withholdings_iva::Entity")]
    TaxWithholdingsIva,
    #[sea_orm(has_many = "super::tax_withholdings_islr::Entity")]
    TaxWithholdingsIslr,
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

impl Related<super::invoice_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InvoiceItems.def()
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

impl Related<super::delivery_notes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DeliveryNotes.def()
    }
}

impl Related<super::tax_withholdings_iva::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaxWithholdingsIva.def()
    }
}

impl Related<super::tax_withholdings_islr::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaxWithholdingsIslr.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
