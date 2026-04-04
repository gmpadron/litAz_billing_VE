//! SeaORM entity for the `company_profiles` table.
//! Stores the fiscal profile of the invoice issuer (emisor).

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "company_profiles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub razon_social: String,
    pub nombre_comercial: Option<String>,
    /// RIF format: [JVEGPC]-XXXXXXXX-X
    #[sea_orm(unique)]
    pub rif: String,
    pub domicilio_fiscal: String,
    pub telefono: String,
    pub email: Option<String>,
    pub es_contribuyente_especial: bool,
    pub nro_contribuyente_especial: Option<String>,
    pub is_active: bool,
    /// UUID of the user who created this record (from JWT)
    pub created_by: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::clients::Entity")]
    Clients,
    #[sea_orm(has_many = "super::invoices::Entity")]
    Invoices,
    #[sea_orm(has_many = "super::credit_notes::Entity")]
    CreditNotes,
    #[sea_orm(has_many = "super::debit_notes::Entity")]
    DebitNotes,
    #[sea_orm(has_many = "super::delivery_notes::Entity")]
    DeliveryNotes,
    #[sea_orm(has_many = "super::numbering_sequences::Entity")]
    NumberingSequences,
    #[sea_orm(has_many = "super::control_number_ranges::Entity")]
    ControlNumberRanges,
    #[sea_orm(has_many = "super::tax_withholdings_iva::Entity")]
    TaxWithholdingsIva,
    #[sea_orm(has_many = "super::tax_withholdings_islr::Entity")]
    TaxWithholdingsIslr,
    #[sea_orm(has_many = "super::purchase_book_entries::Entity")]
    PurchaseBookEntries,
    #[sea_orm(has_many = "super::sales_book_entries::Entity")]
    SalesBookEntries,
}

impl Related<super::clients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Clients.def()
    }
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

impl Related<super::delivery_notes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DeliveryNotes.def()
    }
}

impl Related<super::numbering_sequences::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NumberingSequences.def()
    }
}

impl Related<super::control_number_ranges::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ControlNumberRanges.def()
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

impl Related<super::purchase_book_entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PurchaseBookEntries.def()
    }
}

impl Related<super::sales_book_entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SalesBookEntries.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
