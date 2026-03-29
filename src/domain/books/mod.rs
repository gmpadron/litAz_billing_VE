//! Libros de compras y ventas (mensual, SENIAT).
//!
//! Este modulo contiene la logica de dominio para:
//! - Entradas del Libro de Compras (purchase book)
//! - Entradas del Libro de Ventas (sales book)
//! - Resumen diario de ventas a consumidores finales sin RIF

pub mod daily_summary;
pub mod purchase_book;
pub mod sales_book;

#[allow(unused_imports)]
pub use daily_summary::DailySummary;
#[allow(unused_imports)]
pub use purchase_book::PurchaseBookEntry;
#[allow(unused_imports)]
pub use sales_book::SalesBookEntry;

use thiserror::Error;

/// Errores posibles al validar entradas de libros fiscales.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BookError {
    /// Falta un campo obligatorio.
    #[error("Campo obligatorio faltante: {field}")]
    MissingRequiredField {
        /// Nombre del campo faltante.
        field: String,
    },

    /// El periodo no tiene el formato esperado (YYYY-MM).
    #[error("Formato de periodo invalido: '{value}'. Esperado: YYYY-MM")]
    InvalidPeriodFormat {
        /// Valor proporcionado.
        value: String,
    },

    /// Un monto monetario es negativo cuando no deberia serlo.
    #[error("Monto negativo no permitido en campo '{field}': {value}")]
    NegativeAmount {
        /// Nombre del campo.
        field: String,
        /// Valor proporcionado.
        value: String,
    },

    /// La fecha de la entrada es invalida.
    #[error("Fecha invalida: {0}")]
    InvalidDate(String),

    /// Las bases imponibles no suman el total esperado.
    #[error(
        "Inconsistencia en bases imponibles: la suma de bases ({sum_bases}) no coincide con el total ({total})"
    )]
    BaseMismatch {
        /// Suma de las bases.
        sum_bases: String,
        /// Total declarado.
        total: String,
    },
}
