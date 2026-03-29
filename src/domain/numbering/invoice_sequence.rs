//! Logica de dominio para secuencias de numeros de factura.
//!
//! El numero de factura es un consecutivo interno del contribuyente,
//! secuencial sin saltos. La atomicidad real se garantiza en la capa
//! de servicios (transaccion de base de datos), pero la logica de
//! formato y generacion vive aqui.

use std::fmt;

/// Representa una secuencia de numeros de factura.
///
/// Mantiene el valor actual del consecutivo y un prefijo opcional
/// que se antepone al numero formateado.
#[derive(Debug, Clone)]
pub struct InvoiceSequence {
    /// Valor actual del consecutivo. El proximo numero a emitir sera `current_value + 1`.
    pub current_value: u64,
    /// Prefijo opcional para el numero de factura (ej: "FAC-", "A-").
    pub prefix: Option<String>,
}

impl InvoiceSequence {
    /// Crea una nueva secuencia de factura.
    ///
    /// # Argumentos
    ///
    /// * `current_value` - El ultimo numero emitido (0 si es nueva secuencia).
    /// * `prefix` - Prefijo opcional para anteponer al numero.
    pub fn new(current_value: u64, prefix: Option<String>) -> Self {
        Self {
            current_value,
            prefix,
        }
    }

    /// Genera el siguiente numero de factura en la secuencia.
    ///
    /// Incrementa el contador interno y retorna el numero formateado.
    /// En produccion, esta operacion debe ejecutarse dentro de una
    /// transaccion de base de datos para garantizar atomicidad.
    ///
    /// # Retorna
    ///
    /// El numero de factura formateado como `String`, con el prefijo
    /// (si existe) y el numero rellenado con ceros a 8 digitos.
    pub fn next(&mut self) -> String {
        self.current_value += 1;
        Self::format_number(self.current_value, &self.prefix)
    }

    /// Formatea un numero de factura dado un valor y prefijo opcionales.
    ///
    /// El numero se rellena con ceros a la izquierda hasta 8 digitos.
    /// Si hay prefijo, se antepone al numero.
    ///
    /// # Ejemplos
    ///
    /// ```
    /// use billing_core::domain::numbering::InvoiceSequence;
    ///
    /// assert_eq!(
    ///     InvoiceSequence::format_number(42, &None),
    ///     "00000042"
    /// );
    /// assert_eq!(
    ///     InvoiceSequence::format_number(42, &Some("FAC-".to_string())),
    ///     "FAC-00000042"
    /// );
    /// ```
    pub fn format_number(value: u64, prefix: &Option<String>) -> String {
        match prefix {
            Some(p) => format!("{}{:08}", p, value),
            None => format!("{:08}", value),
        }
    }

    /// Retorna el valor actual del consecutivo (ultimo numero emitido).
    pub fn current(&self) -> u64 {
        self.current_value
    }
}

impl fmt::Display for InvoiceSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "InvoiceSequence(current={}, prefix={:?})",
            self.current_value, self.prefix
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sequence() {
        let seq = InvoiceSequence::new(0, None);
        assert_eq!(seq.current_value, 0);
        assert!(seq.prefix.is_none());
    }

    #[test]
    fn test_next_increments() {
        let mut seq = InvoiceSequence::new(0, None);
        let first = seq.next();
        assert_eq!(first, "00000001");
        assert_eq!(seq.current_value, 1);

        let second = seq.next();
        assert_eq!(second, "00000002");
        assert_eq!(seq.current_value, 2);
    }

    #[test]
    fn test_sequential_no_gaps() {
        let mut seq = InvoiceSequence::new(0, None);
        for i in 1..=100 {
            let num = seq.next();
            assert_eq!(num, format!("{:08}", i));
        }
    }

    #[test]
    fn test_with_prefix() {
        let mut seq = InvoiceSequence::new(0, Some("FAC-".to_string()));
        assert_eq!(seq.next(), "FAC-00000001");
        assert_eq!(seq.next(), "FAC-00000002");
    }

    #[test]
    fn test_format_number_no_prefix() {
        assert_eq!(InvoiceSequence::format_number(1, &None), "00000001");
        assert_eq!(InvoiceSequence::format_number(99999999, &None), "99999999");
    }

    #[test]
    fn test_format_number_with_prefix() {
        let prefix = Some("A-".to_string());
        assert_eq!(InvoiceSequence::format_number(1, &prefix), "A-00000001");
    }

    #[test]
    fn test_format_leading_zeros() {
        assert_eq!(InvoiceSequence::format_number(42, &None), "00000042");
    }

    #[test]
    fn test_start_from_nonzero() {
        let mut seq = InvoiceSequence::new(500, None);
        assert_eq!(seq.next(), "00000501");
    }

    #[test]
    fn test_large_number() {
        let mut seq = InvoiceSequence::new(99999998, None);
        assert_eq!(seq.next(), "99999999");
        // Puede exceder 8 digitos, pero no hay limite en dominio
        assert_eq!(seq.next(), "100000000");
    }

    #[test]
    fn test_current_value() {
        let mut seq = InvoiceSequence::new(0, None);
        assert_eq!(seq.current(), 0);
        seq.next();
        assert_eq!(seq.current(), 1);
    }

    #[test]
    fn test_display() {
        let seq = InvoiceSequence::new(5, Some("X-".to_string()));
        let display = format!("{}", seq);
        assert!(display.contains("5"));
        assert!(display.contains("X-"));
    }
}
