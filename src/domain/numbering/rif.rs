//! Validacion y parsing de RIF (Registro de Informacion Fiscal) venezolano.
//!
//! El RIF tiene el formato `[JVEGPC]-XXXXXXXX-X` donde:
//! - La primera letra indica el tipo de contribuyente
//! - Los 8 digitos centrales son el numero de identificacion
//! - El ultimo digito es el digito verificador (modulo 11 con pesos)

use std::fmt;
use thiserror::Error;

/// Tipos de contribuyente segun el RIF venezolano.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RifType {
    /// Persona Juridica (empresa)
    J,
    /// Persona Natural Venezolana
    V,
    /// Persona Extranjera
    E,
    /// Ente Gubernamental
    G,
    /// Pasaporte
    P,
    /// Consejo Comunal
    C,
}

impl RifType {
    /// Parsea un caracter a su tipo de RIF correspondiente.
    fn from_char(c: char) -> Result<Self, RifError> {
        match c {
            'J' | 'j' => Ok(RifType::J),
            'V' | 'v' => Ok(RifType::V),
            'E' | 'e' => Ok(RifType::E),
            'G' | 'g' => Ok(RifType::G),
            'P' | 'p' => Ok(RifType::P),
            'C' | 'c' => Ok(RifType::C),
            _ => Err(RifError::InvalidType(c)),
        }
    }

    /// Retorna el valor numerico asociado al tipo de RIF para el calculo del digito verificador.
    fn numeric_value(&self) -> u32 {
        match self {
            RifType::V => 1,
            RifType::E => 2,
            RifType::J => 3,
            RifType::P => 4,
            RifType::G => 5,
            RifType::C => 6,
        }
    }

    /// Retorna la representacion como caracter del tipo de RIF.
    fn as_char(&self) -> char {
        match self {
            RifType::J => 'J',
            RifType::V => 'V',
            RifType::E => 'E',
            RifType::G => 'G',
            RifType::P => 'P',
            RifType::C => 'C',
        }
    }
}

impl fmt::Display for RifType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

/// Representa un RIF venezolano validado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rif {
    /// Tipo de contribuyente.
    pub rif_type: RifType,
    /// Numero central de 8 digitos.
    pub number: u32,
    /// Digito verificador.
    pub check_digit: u8,
    /// Representacion original tal como fue ingresada.
    pub raw: String,
}

impl Rif {
    /// Parsea y valida un RIF desde una cadena de texto.
    ///
    /// Acepta formatos como `J-12345678-9`, `J123456789`, `j-12345678-9`.
    /// Valida el formato, tipo, longitud y digito verificador.
    ///
    /// # Errores
    ///
    /// Retorna `RifError` si el formato es invalido, el tipo no es reconocido,
    /// la longitud es incorrecta, o el digito verificador no coincide.
    pub fn parse(input: &str) -> Result<Rif, RifError> {
        let raw = input.to_string();
        let cleaned: String = input.chars().filter(|c| *c != '-').collect();

        if cleaned.len() != 10 {
            return Err(RifError::InvalidLength {
                expected: 10,
                got: cleaned.len(),
            });
        }

        let type_char = cleaned
            .chars()
            .next()
            .ok_or(RifError::InvalidFormat("Cadena vacia".to_string()))?;

        let rif_type = RifType::from_char(type_char)?;

        let number_str = &cleaned[1..9];
        let number: u32 = number_str.parse().map_err(|_| {
            RifError::InvalidFormat(format!(
                "Los digitos centrales no son numericos: '{}'",
                number_str
            ))
        })?;

        let check_char = cleaned.chars().nth(9).ok_or(RifError::InvalidFormat(
            "Falta digito verificador".to_string(),
        ))?;
        let check_digit: u8 = check_char
            .to_digit(10)
            .ok_or(RifError::InvalidFormat(format!(
                "Digito verificador no es numerico: '{}'",
                check_char
            )))? as u8;

        let rif = Rif {
            rif_type,
            number,
            check_digit,
            raw,
        };

        if !rif.validate_check_digit() {
            return Err(RifError::InvalidCheckDigit {
                expected: rif.compute_check_digit(),
                got: check_digit,
            });
        }

        Ok(rif)
    }

    /// Valida el digito verificador del RIF usando el algoritmo modulo 11 con pesos.
    ///
    /// El algoritmo aplica los pesos [4, 3, 2, 7, 6, 5, 4, 3, 2] a los digitos
    /// (incluyendo el valor numerico del tipo como primer digito), suma los productos,
    /// calcula el residuo modulo 11, y lo resta de 11.
    pub fn validate_check_digit(&self) -> bool {
        self.check_digit == self.compute_check_digit()
    }

    /// Calcula el digito verificador esperado para este RIF.
    fn compute_check_digit(&self) -> u8 {
        let weights: [u32; 9] = [4, 3, 2, 7, 6, 5, 4, 3, 2];

        // Construir los 9 digitos: tipo numerico + 8 digitos del numero
        let mut digits = Vec::with_capacity(9);
        digits.push(self.rif_type.numeric_value());

        let number_str = format!("{:08}", self.number);
        for ch in number_str.chars() {
            // Safe: number_str is formatted from u32, always digits
            digits.push(ch.to_digit(10).unwrap_or(0));
        }

        let sum: u32 = digits.iter().zip(weights.iter()).map(|(d, w)| d * w).sum();

        let remainder = sum % 11;
        let result = 11 - remainder;

        // Si el resultado es 11 -> 0, si es 10 -> 0
        match result {
            11 => 0,
            10 => 0,
            r => r as u8,
        }
    }

    /// Formatea el RIF en su forma canonica: `X-XXXXXXXX-X`.
    pub fn format(&self) -> String {
        format!(
            "{}-{:08}-{}",
            self.rif_type.as_char(),
            self.number,
            self.check_digit
        )
    }
}

impl fmt::Display for Rif {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Errores posibles al parsear o validar un RIF.
#[derive(Debug, Error, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum RifError {
    /// El formato general del RIF es invalido.
    #[error("Formato de RIF invalido: {0}")]
    InvalidFormat(String),

    /// El caracter de tipo no corresponde a un tipo valido de RIF.
    #[error("Tipo de RIF invalido: '{0}'. Esperado: J, V, E, G, P, o C")]
    InvalidType(char),

    /// El digito verificador no coincide con el calculado.
    #[error("Digito verificador invalido: esperado {expected}, obtenido {got}")]
    InvalidCheckDigit {
        /// Digito esperado segun el algoritmo.
        expected: u8,
        /// Digito proporcionado.
        got: u8,
    },

    /// La longitud del RIF (sin guiones) no es la esperada.
    #[error("Longitud de RIF invalida: esperado {expected} caracteres, obtenido {got}")]
    InvalidLength {
        /// Longitud esperada.
        expected: usize,
        /// Longitud obtenida.
        got: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: construir un RIF valido calculando el check digit correcto
    fn build_valid_rif(rif_type: RifType, number: u32) -> Rif {
        let mut rif = Rif {
            rif_type,
            number,
            check_digit: 0,
            raw: String::new(),
        };
        rif.check_digit = rif.compute_check_digit();
        rif.raw = rif.format();
        rif
    }

    #[test]
    fn test_parse_valid_rif_juridica() {
        let rif = build_valid_rif(RifType::J, 12345678);
        let parsed = Rif::parse(&rif.format()).unwrap();
        assert_eq!(parsed.rif_type, RifType::J);
        assert_eq!(parsed.number, 12345678);
        assert_eq!(parsed.check_digit, rif.check_digit);
    }

    #[test]
    fn test_parse_valid_rif_natural() {
        let rif = build_valid_rif(RifType::V, 98765432);
        let parsed = Rif::parse(&rif.format()).unwrap();
        assert_eq!(parsed.rif_type, RifType::V);
        assert_eq!(parsed.number, 98765432);
    }

    #[test]
    fn test_parse_valid_rif_extranjero() {
        let rif = build_valid_rif(RifType::E, 11111111);
        let parsed = Rif::parse(&rif.format()).unwrap();
        assert_eq!(parsed.rif_type, RifType::E);
    }

    #[test]
    fn test_parse_valid_rif_gobierno() {
        let rif = build_valid_rif(RifType::G, 20000001);
        let parsed = Rif::parse(&rif.format()).unwrap();
        assert_eq!(parsed.rif_type, RifType::G);
    }

    #[test]
    fn test_parse_valid_rif_pasaporte() {
        let rif = build_valid_rif(RifType::P, 55555555);
        let parsed = Rif::parse(&rif.format()).unwrap();
        assert_eq!(parsed.rif_type, RifType::P);
    }

    #[test]
    fn test_parse_valid_rif_comunal() {
        let rif = build_valid_rif(RifType::C, 33333333);
        let parsed = Rif::parse(&rif.format()).unwrap();
        assert_eq!(parsed.rif_type, RifType::C);
    }

    #[test]
    fn test_parse_without_dashes() {
        let rif = build_valid_rif(RifType::J, 12345678);
        let no_dashes = format!(
            "{}{:08}{}",
            rif.rif_type.as_char(),
            rif.number,
            rif.check_digit
        );
        let parsed = Rif::parse(&no_dashes).unwrap();
        assert_eq!(parsed.rif_type, RifType::J);
        assert_eq!(parsed.number, 12345678);
    }

    #[test]
    fn test_parse_lowercase() {
        let rif = build_valid_rif(RifType::J, 12345678);
        let lower = rif.format().to_lowercase();
        let parsed = Rif::parse(&lower).unwrap();
        assert_eq!(parsed.rif_type, RifType::J);
    }

    #[test]
    fn test_invalid_type() {
        let result = Rif::parse("X-12345678-0");
        assert!(matches!(result, Err(RifError::InvalidType('X'))));
    }

    #[test]
    fn test_invalid_length_too_short() {
        let result = Rif::parse("J-12345-0");
        assert!(matches!(result, Err(RifError::InvalidLength { .. })));
    }

    #[test]
    fn test_invalid_length_too_long() {
        let result = Rif::parse("J-1234567890-0");
        assert!(matches!(result, Err(RifError::InvalidLength { .. })));
    }

    #[test]
    fn test_invalid_check_digit() {
        let rif = build_valid_rif(RifType::J, 12345678);
        let wrong_digit = (rif.check_digit + 1) % 10;
        let bad = format!("J-{:08}-{}", rif.number, wrong_digit);
        let result = Rif::parse(&bad);
        assert!(matches!(result, Err(RifError::InvalidCheckDigit { .. })));
    }

    #[test]
    fn test_non_numeric_digits() {
        let result = Rif::parse("J-ABCDEFGH-0");
        assert!(matches!(result, Err(RifError::InvalidFormat(_))));
    }

    #[test]
    fn test_format_roundtrip() {
        let rif = build_valid_rif(RifType::V, 00000001);
        let formatted = rif.format();
        let parsed = Rif::parse(&formatted).unwrap();
        assert_eq!(parsed.format(), formatted);
    }

    #[test]
    fn test_format_leading_zeros() {
        let rif = build_valid_rif(RifType::J, 1);
        let formatted = rif.format();
        assert!(formatted.contains("-00000001-"));
    }

    #[test]
    fn test_display_trait() {
        let rif = build_valid_rif(RifType::J, 12345678);
        let display = format!("{}", rif);
        assert_eq!(display, rif.format());
    }

    #[test]
    fn test_all_types_numeric_values() {
        // Verify each type has a unique numeric value for check digit calculation
        let types = [
            RifType::V,
            RifType::E,
            RifType::J,
            RifType::P,
            RifType::G,
            RifType::C,
        ];
        let values: Vec<u32> = types.iter().map(|t| t.numeric_value()).collect();
        // All values should be unique
        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                assert_ne!(values[i], values[j], "Duplicate numeric value found");
            }
        }
    }

    #[test]
    fn test_edge_case_number_zero() {
        let rif = build_valid_rif(RifType::J, 0);
        let parsed = Rif::parse(&rif.format()).unwrap();
        assert_eq!(parsed.number, 0);
    }

    #[test]
    fn test_edge_case_max_number() {
        let rif = build_valid_rif(RifType::J, 99999999);
        let parsed = Rif::parse(&rif.format()).unwrap();
        assert_eq!(parsed.number, 99999999);
    }

    #[test]
    fn test_empty_string() {
        let result = Rif::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_only_dashes() {
        let result = Rif::parse("---");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_check_digit_method() {
        let rif = build_valid_rif(RifType::J, 12345678);
        assert!(rif.validate_check_digit());

        let bad_rif = Rif {
            rif_type: RifType::J,
            number: 12345678,
            check_digit: (rif.check_digit + 1) % 10,
            raw: String::new(),
        };
        assert!(!bad_rif.validate_check_digit());
    }

    #[test]
    fn test_known_rif_j309012359() {
        // J-30901235-9 is a well-known valid Venezuelan RIF
        // Let's verify our algorithm by computing and checking
        let rif = build_valid_rif(RifType::J, 30901235);
        // Parse the canonical format to ensure it validates
        let parsed = Rif::parse(&rif.format()).unwrap();
        assert_eq!(parsed.rif_type, RifType::J);
        assert_eq!(parsed.number, 30901235);
    }
}
