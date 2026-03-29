//! Secuencias atomicas de numero de factura, numero de control, y validacion de RIF.
//!
//! Este modulo contiene la logica de dominio pura para:
//! - Validacion y parsing de RIF venezolano (formato SENIAT)
//! - Secuencias de numeracion de facturas
//! - Rangos de numeros de control (asignados por imprenta autorizada)

pub mod control_number;
pub mod invoice_sequence;
pub mod rif;

#[allow(unused_imports)]
pub use control_number::{ControlNumberError, ControlNumberRange};
#[allow(unused_imports)]
pub use invoice_sequence::InvoiceSequence;
#[allow(unused_imports)]
pub use rif::{Rif, RifError, RifType};
