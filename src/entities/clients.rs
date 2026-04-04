//! SeaORM entity for the `clients` table.
//! Stores customers (buyers). RIF is optional for consumer final (persona natural sin RIF).

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "clients")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub razon_social: String,
    pub nombre_comercial: Option<String>,
    /// RIF format: [JVEGPC]-XXXXXXXX-X. Nullable for consumer final.
    pub rif: Option<String>,
    /// National ID for natural persons without RIF
    pub cedula: Option<String>,
    pub domicilio_fiscal: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    /// If true, "SIN DERECHO A CRÉDITO FISCAL" must appear on invoices
    pub es_consumidor_final: bool,
    pub es_contribuyente_especial: bool,
    pub is_active: bool,
    /// Empresa a la que pertenece este cliente
    pub company_profile_id: Uuid,
    /// UUID of the user who created this record (from JWT)
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
    #[sea_orm(has_many = "super::invoices::Entity")]
    Invoices,
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

impl Related<super::company_profiles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CompanyProfile.def()
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
