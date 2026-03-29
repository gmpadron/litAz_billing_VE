// Lógica de dominio para facturas, notas de crédito, notas de débito y guías de despacho

pub mod builder;
pub mod credit_note;
pub mod debit_note;
pub mod delivery_note;
pub mod status;
pub mod validation;

#[allow(unused_imports)]
pub use builder::{InvoiceBuilder, InvoiceData, InvoiceItemData, InvoiceTotals, PaymentCondition};
#[allow(unused_imports)]
pub use credit_note::{CreditNoteBuilder, CreditNoteData, CreditNoteError, CreditNoteItemData};
#[allow(unused_imports)]
pub use debit_note::{DebitNoteBuilder, DebitNoteData, DebitNoteError, DebitNoteItemData};
#[allow(unused_imports)]
pub use delivery_note::{
    DeliveryNoteBuilder, DeliveryNoteData, DeliveryNoteError, DeliveryNoteItemData,
};
#[allow(unused_imports)]
pub use status::{DocumentStatus, StatusTransitionError};
#[allow(unused_imports)]
pub use validation::ValidationError;
