//! SeaORM entity for the `tax_withholdings_iva` table.
//! IVA withholdings — 75% or 100% of IVA, quincenal declaration.

use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tax_withholdings_iva")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Withholding voucher number (comprobante de retención IVA)
    #[sea_orm(unique)]
    pub voucher_number: String,
    pub invoice_id: Uuid,
    /// The supplier being withheld from
    pub supplier_id: Uuid,
    pub company_profile_id: Uuid,
    pub withholding_date: DateTimeWithTimeZone,
    /// PRIMERA (days 1-15) | SEGUNDA (days 16-end)
    pub period: String,
    /// Year-month for reporting: "2026-03"
    pub reporting_period: String,
    /// IVA amount on the invoice
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub iva_amount: Decimal,
    /// Withholding percentage: 75.00 or 100.00
    #[sea_orm(column_type = "Decimal(Some((5, 2)))")]
    pub withholding_percentage: Decimal,
    /// Withheld amount = iva_amount * (percentage / 100)
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub withheld_amount: Decimal,
    /// Emitido | Declarado | Anulado
    pub status: String,
    /// Path/reference to generated XML file for SENIAT portal upload
    pub xml_file_path: Option<String>,
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
    #[sea_orm(
        belongs_to = "super::clients::Entity",
        from = "Column::SupplierId",
        to = "super::clients::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    Supplier,
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
        Relation::Supplier.def()
    }
}

impl Related<super::company_profiles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CompanyProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
