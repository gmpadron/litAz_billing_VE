use sea_orm_migration::prelude::*;

/// Elimina el estado "Anulada" de todos los documentos fiscales.
///
/// Según la PA SNAT/2011/0071, ningún documento fiscal se anula:
/// - Facturas: se corrigen con Nota de Crédito
/// - Notas de Crédito: se compensan con Nota de Débito
/// - Notas de Débito: se compensan con Nota de Crédito
/// - Guías de despacho: se corrigen emitiendo una nueva
///
/// Cambios en `invoices`:
/// - Reemplaza CHECK constraint para permitir solo 'Emitida'
/// - Elimina la columna `annulment_reason`
///
/// Cambios en `credit_notes` y `debit_notes`:
/// - Reemplaza CHECK constraint para permitir solo 'Emitida'
///
/// Cambios en `delivery_notes`:
/// - Reemplaza CHECK constraint para permitir solo 'Emitida' y 'Entregada'
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240001_000018_remove_invoice_annulment"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── invoices ──────────────────────────────────────────────────────────
        db.execute_unprepared(
            "ALTER TABLE invoices DROP CONSTRAINT IF EXISTS chk_invoice_status",
        )
        .await?;

        // Normalizar filas existentes: cualquier status distinto de 'Emitida'
        // (ej: 'Anulada') se convierte a 'Emitida' antes de agregar el constraint.
        db.execute_unprepared(
            "UPDATE invoices SET status = 'Emitida' WHERE status != 'Emitida'",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE invoices ADD CONSTRAINT chk_invoice_status \
             CHECK (status IN ('Emitida'))",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE invoices DROP COLUMN IF EXISTS annulment_reason",
        )
        .await?;

        // ── credit_notes ──────────────────────────────────────────────────────
        db.execute_unprepared(
            "ALTER TABLE credit_notes DROP CONSTRAINT IF EXISTS chk_credit_note_status",
        )
        .await?;

        db.execute_unprepared(
            "UPDATE credit_notes SET status = 'Emitida' WHERE status != 'Emitida'",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE credit_notes ADD CONSTRAINT chk_credit_note_status \
             CHECK (status IN ('Emitida'))",
        )
        .await?;

        // ── debit_notes ───────────────────────────────────────────────────────
        db.execute_unprepared(
            "ALTER TABLE debit_notes DROP CONSTRAINT IF EXISTS chk_debit_note_status",
        )
        .await?;

        db.execute_unprepared(
            "UPDATE debit_notes SET status = 'Emitida' WHERE status != 'Emitida'",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE debit_notes ADD CONSTRAINT chk_debit_note_status \
             CHECK (status IN ('Emitida'))",
        )
        .await?;

        // ── delivery_notes ────────────────────────────────────────────────────
        // Delivery notes can be 'Emitida' (issued) or 'Entregada' (delivered).
        // 'Anulada' is removed — a delivery note is never voided.
        db.execute_unprepared(
            "ALTER TABLE delivery_notes DROP CONSTRAINT IF EXISTS chk_delivery_note_status",
        )
        .await?;

        db.execute_unprepared(
            "UPDATE delivery_notes SET status = 'Emitida' \
             WHERE status NOT IN ('Emitida', 'Entregada')",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE delivery_notes ADD CONSTRAINT chk_delivery_note_status \
             CHECK (status IN ('Emitida', 'Entregada'))",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Restore invoices
        db.execute_unprepared(
            "ALTER TABLE invoices DROP CONSTRAINT IF EXISTS chk_invoice_status",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE invoices ADD CONSTRAINT chk_invoice_status \
             CHECK (status IN ('Emitida', 'Anulada'))",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS annulment_reason TEXT",
        )
        .await?;

        // Restore credit_notes
        db.execute_unprepared(
            "ALTER TABLE credit_notes DROP CONSTRAINT IF EXISTS chk_credit_note_status",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE credit_notes ADD CONSTRAINT chk_credit_note_status \
             CHECK (status IN ('Emitida', 'Anulada'))",
        )
        .await?;

        // Restore debit_notes
        db.execute_unprepared(
            "ALTER TABLE debit_notes DROP CONSTRAINT IF EXISTS chk_debit_note_status",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE debit_notes ADD CONSTRAINT chk_debit_note_status \
             CHECK (status IN ('Emitida', 'Anulada'))",
        )
        .await?;

        // Restore delivery_notes
        db.execute_unprepared(
            "ALTER TABLE delivery_notes DROP CONSTRAINT IF EXISTS chk_delivery_note_status",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE delivery_notes ADD CONSTRAINT chk_delivery_note_status \
             CHECK (status IN ('Emitida', 'Anulada', 'Entregada'))",
        )
        .await?;

        Ok(())
    }
}
