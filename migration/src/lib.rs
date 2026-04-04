pub use sea_orm_migration::prelude::*;

mod m20240001_000001_create_company_profiles;
mod m20240001_000002_create_clients;
mod m20240001_000003_create_exchange_rates;
mod m20240001_000004_create_numbering_sequences;
mod m20240001_000005_create_control_number_ranges;
mod m20240001_000006_create_invoices;
mod m20240001_000007_create_invoice_items;
mod m20240001_000008_create_credit_notes;
mod m20240001_000009_create_debit_notes;
mod m20240001_000010_create_delivery_notes;
mod m20240001_000011_create_tax_withholdings_iva;
mod m20240001_000012_create_tax_withholdings_islr;
mod m20240001_000013_create_purchase_book_entries;
mod m20240001_000014_create_sales_book_entries;
mod m20240001_000015_create_audit_logs;
mod m20240001_000016_fix_invoice_items_created_by_purchase_rif_arc_unique;
mod m20240001_000017_add_fk_invoice_id_book_entries;
mod m20240001_000018_remove_invoice_annulment;
mod m20240001_000019_fix_arc_sequence_and_audit_actions;
mod m20240001_000020_add_igtf_to_invoices;
mod m20240001_000021_add_multi_company_support;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240001_000001_create_company_profiles::Migration),
            Box::new(m20240001_000002_create_clients::Migration),
            Box::new(m20240001_000003_create_exchange_rates::Migration),
            Box::new(m20240001_000004_create_numbering_sequences::Migration),
            Box::new(m20240001_000005_create_control_number_ranges::Migration),
            Box::new(m20240001_000006_create_invoices::Migration),
            Box::new(m20240001_000007_create_invoice_items::Migration),
            Box::new(m20240001_000008_create_credit_notes::Migration),
            Box::new(m20240001_000009_create_debit_notes::Migration),
            Box::new(m20240001_000010_create_delivery_notes::Migration),
            Box::new(m20240001_000011_create_tax_withholdings_iva::Migration),
            Box::new(m20240001_000012_create_tax_withholdings_islr::Migration),
            Box::new(m20240001_000013_create_purchase_book_entries::Migration),
            Box::new(m20240001_000014_create_sales_book_entries::Migration),
            Box::new(m20240001_000015_create_audit_logs::Migration),
            Box::new(m20240001_000016_fix_invoice_items_created_by_purchase_rif_arc_unique::Migration),
            Box::new(m20240001_000017_add_fk_invoice_id_book_entries::Migration),
            Box::new(m20240001_000018_remove_invoice_annulment::Migration),
            Box::new(m20240001_000019_fix_arc_sequence_and_audit_actions::Migration),
            Box::new(m20240001_000020_add_igtf_to_invoices::Migration),
            Box::new(m20240001_000021_add_multi_company_support::Migration),
        ]
    }
}
