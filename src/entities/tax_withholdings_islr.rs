//! SeaORM entity for the `tax_withholdings_islr` table.
//! ISLR withholdings — variable rate by activity, monthly declaration.
//! Generates ARC (Comprobante de Retención) for the supplier.

use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tax_withholdings_islr")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// ARC (Comprobante de Retención ISLR) number
    #[sea_orm(unique)]
    pub arc_number: String,
    pub invoice_id: Uuid,
    pub supplier_id: Uuid,
    pub company_profile_id: Uuid,
    pub withholding_date: DateTimeWithTimeZone,
    /// Year-month for monthly declaration: "2026-03"
    pub reporting_period: String,
    /// Activity code per Decreto 1.808
    pub activity_code: String,
    pub activity_description: String,
    /// Gross amount subject to withholding
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub taxable_amount: Decimal,
    /// Withholding rate (%) e.g. 1.00, 2.00, 3.00, 5.00, 34.00
    #[sea_orm(column_type = "Decimal(Some((5, 2)))")]
    pub withholding_rate: Decimal,
    /// Withheld amount = taxable_amount * (rate / 100)
    #[sea_orm(column_type = "Decimal(Some((18, 2)))")]
    pub withheld_amount: Decimal,
    /// Emitido | Declarado | Anulado
    pub status: String,
    /// Path/reference to generated TXT file for SENIAT portal upload
    pub txt_file_path: Option<String>,
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
