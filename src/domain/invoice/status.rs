//! Estados de documentos fiscales y transiciones válidas.
//!
//! Según la PA SNAT/2011/0071 y PA SNAT/2024/000102, las facturas emitidas
//! NUNCA se anulan. Toda corrección se realiza mediante Nota de Crédito.

use thiserror::Error;

/// Estado de un documento fiscal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentStatus {
    /// Borrador: el documento fue creado pero aún no emitido.
    Draft,
    /// Emitida: el documento fue emitido y tiene validez fiscal. Es inmutable.
    Issued,
}

/// Error en transición de estado de documento.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StatusTransitionError {
    #[error("Transición de estado inválida: {from:?} -> {to:?}")]
    InvalidTransition {
        from: DocumentStatus,
        to: DocumentStatus,
    },
}

impl DocumentStatus {
    /// Verifica si la transición al estado destino es válida.
    ///
    /// Transiciones permitidas:
    /// - `Draft` → `Issued`
    ///
    /// Una factura emitida no puede cambiar de estado. Para corregirla
    /// se debe emitir una Nota de Crédito que la referencie.
    pub fn can_transition_to(&self, target: &DocumentStatus) -> bool {
        matches!(
            (self, target),
            (DocumentStatus::Draft, DocumentStatus::Issued)
        )
    }

    /// Intenta realizar la transición, retornando error si no es válida.
    pub fn transition_to(
        &self,
        target: DocumentStatus,
    ) -> Result<DocumentStatus, StatusTransitionError> {
        if self.can_transition_to(&target) {
            Ok(target)
        } else {
            Err(StatusTransitionError::InvalidTransition {
                from: *self,
                to: target,
            })
        }
    }
}

impl std::fmt::Display for DocumentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentStatus::Draft => write!(f, "Borrador"),
            DocumentStatus::Issued => write!(f, "Emitida"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draft_to_issued_allowed() {
        assert!(DocumentStatus::Draft.can_transition_to(&DocumentStatus::Issued));
    }

    #[test]
    fn test_issued_to_draft_not_allowed() {
        assert!(!DocumentStatus::Issued.can_transition_to(&DocumentStatus::Draft));
    }

    #[test]
    fn test_issued_to_issued_not_allowed() {
        assert!(!DocumentStatus::Issued.can_transition_to(&DocumentStatus::Issued));
    }

    #[test]
    fn test_draft_to_draft_not_allowed() {
        assert!(!DocumentStatus::Draft.can_transition_to(&DocumentStatus::Draft));
    }

    #[test]
    fn test_transition_to_returns_new_state() {
        let result = DocumentStatus::Draft.transition_to(DocumentStatus::Issued);
        assert_eq!(result, Ok(DocumentStatus::Issued));
    }

    #[test]
    fn test_transition_to_returns_error_on_invalid() {
        let result = DocumentStatus::Issued.transition_to(DocumentStatus::Draft);
        assert!(matches!(
            result,
            Err(StatusTransitionError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", DocumentStatus::Draft), "Borrador");
        assert_eq!(format!("{}", DocumentStatus::Issued), "Emitida");
    }
}
