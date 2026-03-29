//! SeaORM entities for all database tables.
//! These are the database-layer models — do NOT derive Serialize on them directly.
//! Use DTOs in src/dto/ for API request/response structs.

pub mod audit_logs;
pub mod clients;
pub mod company_profiles;
pub mod control_number_ranges;
pub mod credit_notes;
pub mod debit_notes;
pub mod delivery_notes;
pub mod exchange_rates;
pub mod invoice_items;
pub mod invoices;
pub mod numbering_sequences;
pub mod purchase_book_entries;
pub mod sales_book_entries;
pub mod tax_withholdings_islr;
pub mod tax_withholdings_iva;
