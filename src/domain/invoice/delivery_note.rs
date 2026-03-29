//! Lógica de dominio para Órdenes de Entrega / Guías de Despacho.
//!
//! Las guías de despacho documentan el traslado de mercancía. Pueden referenciar
//! una factura (opcional).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;

/// Error específico de guías de despacho.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DeliveryNoteError {
    #[error("La guía de despacho debe tener al menos un ítem")]
    NoItems,

    #[error("El nombre del destinatario es obligatorio")]
    MissingRecipientName,

    #[error("La dirección de destino es obligatoria")]
    MissingDestinationAddress,

    #[error("El ítem {0} tiene descripción vacía")]
    ItemEmptyDescription(usize),

    #[error("El ítem {0} tiene cantidad inválida (debe ser > 0)")]
    ItemInvalidQuantity(usize),
}

/// Ítem de una guía de despacho (bien físico).
#[derive(Debug, Clone)]
pub struct DeliveryNoteItemData {
    /// Descripción del bien.
    pub description: String,
    /// Cantidad despachada.
    pub quantity: Decimal,
    /// Unidad de medida (ej: "unidades", "kg", "litros").
    pub unit: String,
}

/// Datos de una guía de despacho (entidad de dominio puro).
#[derive(Debug, Clone)]
pub struct DeliveryNoteData {
    /// Número de la guía de despacho.
    pub delivery_note_number: String,
    /// Número de factura relacionada (opcional).
    pub related_invoice_number: Option<String>,
    /// Fecha de emisión.
    pub issue_date: NaiveDate,
    /// Nombre o razón social del destinatario.
    pub recipient_name: String,
    /// RIF del destinatario (opcional).
    pub recipient_rif: Option<String>,
    /// Dirección de destino.
    pub destination_address: String,
    /// Placa o identificación del vehículo de transporte (opcional).
    pub vehicle_info: Option<String>,
    /// Nombre del conductor (opcional).
    pub driver_name: Option<String>,
    /// Ítems despachados.
    pub items: Vec<DeliveryNoteItemData>,
}

/// Constructor de guías de despacho con patrón Builder.
#[derive(Debug, Default)]
pub struct DeliveryNoteBuilder {
    delivery_note_number: String,
    related_invoice_number: Option<String>,
    issue_date: Option<NaiveDate>,
    recipient_name: String,
    recipient_rif: Option<String>,
    destination_address: String,
    vehicle_info: Option<String>,
    driver_name: Option<String>,
    items: Vec<DeliveryNoteItemData>,
}

impl DeliveryNoteBuilder {
    /// Crea un nuevo builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Establece el número de la guía de despacho.
    pub fn delivery_note_number(mut self, number: impl Into<String>) -> Self {
        self.delivery_note_number = number.into();
        self
    }

    /// Referencia a la factura relacionada (opcional).
    pub fn related_invoice(mut self, invoice_number: impl Into<String>) -> Self {
        self.related_invoice_number = Some(invoice_number.into());
        self
    }

    /// Establece la fecha de emisión.
    pub fn issue_date(mut self, date: NaiveDate) -> Self {
        self.issue_date = Some(date);
        self
    }

    /// Establece los datos del destinatario.
    pub fn recipient(
        mut self,
        name: impl Into<String>,
        rif: Option<impl Into<String>>,
        address: impl Into<String>,
    ) -> Self {
        self.recipient_name = name.into();
        self.recipient_rif = rif.map(|r| r.into());
        self.destination_address = address.into();
        self
    }

    /// Establece la información del vehículo de transporte.
    pub fn vehicle(mut self, vehicle_info: impl Into<String>, driver: impl Into<String>) -> Self {
        self.vehicle_info = Some(vehicle_info.into());
        self.driver_name = Some(driver.into());
        self
    }

    /// Agrega un ítem a la guía de despacho.
    pub fn add_item(
        mut self,
        description: impl Into<String>,
        quantity: Decimal,
        unit: impl Into<String>,
    ) -> Self {
        self.items.push(DeliveryNoteItemData {
            description: description.into(),
            quantity,
            unit: unit.into(),
        });
        self
    }

    /// Construye la guía de despacho con todas las validaciones.
    pub fn build(self) -> Result<DeliveryNoteData, Vec<DeliveryNoteError>> {
        let mut errors = Vec::new();

        if self.recipient_name.trim().is_empty() {
            errors.push(DeliveryNoteError::MissingRecipientName);
        }

        if self.destination_address.trim().is_empty() {
            errors.push(DeliveryNoteError::MissingDestinationAddress);
        }

        if self.items.is_empty() {
            errors.push(DeliveryNoteError::NoItems);
        } else {
            for (i, item) in self.items.iter().enumerate() {
                if item.description.trim().is_empty() {
                    errors.push(DeliveryNoteError::ItemEmptyDescription(i));
                }
                if item.quantity <= Decimal::ZERO {
                    errors.push(DeliveryNoteError::ItemInvalidQuantity(i));
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let date = self
            .issue_date
            .unwrap_or_else(|| chrono::Local::now().date_naive());

        Ok(DeliveryNoteData {
            delivery_note_number: self.delivery_note_number,
            related_invoice_number: self.related_invoice_number,
            issue_date: date,
            recipient_name: self.recipient_name,
            recipient_rif: self.recipient_rif,
            destination_address: self.destination_address,
            vehicle_info: self.vehicle_info,
            driver_name: self.driver_name,
            items: self.items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, 26).unwrap()
    }

    #[test]
    fn test_build_valid_delivery_note() {
        let note = DeliveryNoteBuilder::new()
            .delivery_note_number("GD-0001")
            .issue_date(sample_date())
            .recipient(
                "Cliente Destino C.A.",
                Some("J-98765432-0"),
                "Calle Sur, Valencia",
            )
            .add_item("Cajas de producto", dec!(10), "unidades")
            .build();

        assert!(note.is_ok(), "Expected Ok, got: {:?}", note.err());
    }

    #[test]
    fn test_delivery_note_with_invoice_reference() {
        let note = DeliveryNoteBuilder::new()
            .delivery_note_number("GD-0002")
            .related_invoice("00000005")
            .issue_date(sample_date())
            .recipient("Cliente C.A.", Some("J-11111111-1"), "Av. Bolívar, Maracay")
            .vehicle("AB-123-CD", "Juan Pérez")
            .add_item("Producto A", dec!(5), "kg")
            .build()
            .unwrap();

        assert_eq!(note.related_invoice_number, Some("00000005".to_string()));
        assert!(note.vehicle_info.is_some());
        assert!(note.driver_name.is_some());
    }

    #[test]
    fn test_delivery_note_without_invoice_reference() {
        let note = DeliveryNoteBuilder::new()
            .delivery_note_number("GD-0003")
            .issue_date(sample_date())
            .recipient(
                "Almacén Central",
                None::<String>,
                "Zona Industrial, Guacara",
            )
            .add_item("Paleta de materiales", dec!(2), "paletas")
            .build()
            .unwrap();

        assert!(note.related_invoice_number.is_none());
    }

    #[test]
    fn test_delivery_note_missing_recipient() {
        let result = DeliveryNoteBuilder::new()
            .delivery_note_number("GD-0001")
            .issue_date(sample_date())
            .recipient("", None::<String>, "Av. Principal")
            .add_item("Producto", dec!(1), "unidades")
            .build();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains(&DeliveryNoteError::MissingRecipientName)
        );
    }

    #[test]
    fn test_delivery_note_missing_destination() {
        let result = DeliveryNoteBuilder::new()
            .delivery_note_number("GD-0001")
            .issue_date(sample_date())
            .recipient("Cliente C.A.", None::<String>, "")
            .add_item("Producto", dec!(1), "unidades")
            .build();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains(&DeliveryNoteError::MissingDestinationAddress)
        );
    }

    #[test]
    fn test_delivery_note_no_items() {
        let result = DeliveryNoteBuilder::new()
            .delivery_note_number("GD-0001")
            .issue_date(sample_date())
            .recipient("Cliente C.A.", None::<String>, "Av. Principal")
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains(&DeliveryNoteError::NoItems));
    }
}
