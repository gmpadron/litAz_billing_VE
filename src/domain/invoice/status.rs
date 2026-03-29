//! Estados de documentos fiscales y transiciones válidas.

use thiserror::Error;

/// Estado de un documento fiscal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentStatus {
    /// Borrador: el documento fue creado pero aún no emitido.
    Draft,
    /// Emitida: el documento fue emitido y tiene validez fiscal.
    Issued,
    /// Anulada: el documento fue anulado. Permanece en el sistema pero sin efecto fiscal.
    Voided,
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
    /// - `Issued` → `Voided`
    ///
    /// No se permite ninguna otra transición.
    pub fn can_transition_to(&self, target: &DocumentStatus) -> bool {
        matches!(
            (self, target),
            (DocumentStatus::Draft, DocumentStatus::Issued)
                | (DocumentStatus::Issued, DocumentStatus::Voided)
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
            DocumentStatus::Voided => write!(f, "Anulada"),
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
    fn test_issued_to_voided_allowed() {
        assert!(DocumentStatus::Issued.can_transition_to(&DocumentStatus::Voided));
    }

    #[test]
    fn test_draft_to_voided_not_allowed() {
        assert!(!DocumentStatus::Draft.can_transition_to(&DocumentStatus::Voided));
    }

    #[test]
    fn test_issued_to_draft_not_allowed() {
        assert!(!DocumentStatus::Issued.can_transition_to(&DocumentStatus::Draft));
    }

    #[test]
    fn test_voided_transitions_not_allowed() {
        assert!(!DocumentStatus::Voided.can_transition_to(&DocumentStatus::Draft));
        assert!(!DocumentStatus::Voided.can_transition_to(&DocumentStatus::Issued));
        assert!(!DocumentStatus::Voided.can_transition_to(&DocumentStatus::Voided));
    }

    #[test]
    fn test_same_state_not_allowed() {
        assert!(!DocumentStatus::Draft.can_transition_to(&DocumentStatus::Draft));
        assert!(!DocumentStatus::Issued.can_transition_to(&DocumentStatus::Issued));
    }

    #[test]
    fn test_transition_to_returns_new_state() {
        let result = DocumentStatus::Draft.transition_to(DocumentStatus::Issued);
        assert_eq!(result, Ok(DocumentStatus::Issued));
    }

    #[test]
    fn test_transition_to_returns_error_on_invalid() {
        let result = DocumentStatus::Draft.transition_to(DocumentStatus::Voided);
        assert!(matches!(
            result,
            Err(StatusTransitionError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", DocumentStatus::Draft), "Borrador");
        assert_eq!(format!("{}", DocumentStatus::Issued), "Emitida");
        assert_eq!(format!("{}", DocumentStatus::Voided), "Anulada");
    }
}
