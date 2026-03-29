//! SeaORM entity for the `invoice_items` table.
//! Line items belonging to a single invoice.

use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "invoice_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub invoice_id: Uuid,
    /// Ordering within the invoice
    pub line_number: i32,
    pub description: String,
    #[sea_orm(column_type = "Decimal(Some((18, 4)))")]
    pub quantity: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub unit_price: Decimal,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub discount: Decimal,
    /// Net subtotal = (quantity * unit_price) - discount
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub subtotal: Decimal,
    /// IVA rate as percentage e.g. 16.00, 8.00, 0.00, 31.00
    #[sea_orm(column_type = "Decimal(Some((5, 2)))")]
    pub tax_rate: Decimal,
    /// GENERAL | REDUCIDA | LUJO | EXENTO
    pub tax_type: String,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub tax_amount: Decimal,
    /// Line total = subtotal + tax_amount
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub total: Decimal,
    pub product_code: Option<String>,
    /// UUID of the user who created this line item (from JWT payload)
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
        on_delete = "Restrict"
    )]
    Invoice,
}

impl Related<super::invoices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Invoice.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
