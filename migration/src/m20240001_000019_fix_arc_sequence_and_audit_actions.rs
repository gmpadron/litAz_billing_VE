use sea_orm_migration::prelude::*;

/// Correcciones de constraints e integridad:
///
/// 1. `chk_audit_action` — elimina 'ANNUL' y 'DELETE' (ningún documento fiscal
///    se anula ni se borra según PA SNAT/2011/0071).
///
/// 2. `chk_sequence_type` — agrega 'ARC' para que los comprobantes de retención
///    ISLR usen la misma tabla de secuencias que los demás documentos fiscales,
///    garantizando atomicidad y sin race conditions.
///
/// 3. Inserta una fila ARC en `numbering_sequences` para cada empresa activa
///    que todavía no tenga secuencia ARC.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240001_000019_fix_arc_sequence_and_audit_actions"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── 1. audit_logs: quitar 'ANNUL' y 'DELETE' ─────────────────────────
        // Los documentos fiscales son inmutables. No existe anulación ni borrado.
        db.execute_unprepared(
            "ALTER TABLE audit_logs DROP CONSTRAINT IF EXISTS chk_audit_action",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE audit_logs ADD CONSTRAINT chk_audit_action \
             CHECK (action IN ('CREATE', 'UPDATE', 'VIEW', 'EXPORT', 'DECLARE'))",
        )
        .await?;

        // ── 2. numbering_sequences: agregar 'ARC' al constraint ───────────────
        db.execute_unprepared(
            "ALTER TABLE numbering_sequences DROP CONSTRAINT IF EXISTS chk_sequence_type",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE numbering_sequences ADD CONSTRAINT chk_sequence_type \
             CHECK (sequence_type IN ('INVOICE', 'CREDIT_NOTE', 'DEBIT_NOTE', 'DELIVERY_NOTE', 'ARC'))",
        )
        .await?;

        // ── 3. Insertar secuencia ARC para empresas activas que no la tengan ──
        // Usa el mismo created_by = UUID cero (system) que el seeder original.
        db.execute_unprepared(
            r#"
            INSERT INTO numbering_sequences (
                id, sequence_type, prefix, current_value, min_value, max_value,
                is_active, company_profile_id, created_by, created_at, updated_at
            )
            SELECT
                gen_random_uuid(),
                'ARC',
                'ARC-',
                0,
                1,
                NULL,
                true,
                cp.id,
                '00000000-0000-0000-0000-000000000000'::uuid,
                NOW(),
                NOW()
            FROM company_profiles cp
            WHERE cp.is_active = true
              AND NOT EXISTS (
                  SELECT 1 FROM numbering_sequences ns
                  WHERE ns.company_profile_id = cp.id
                    AND ns.sequence_type = 'ARC'
              )
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Restaurar audit_logs constraint original
        db.execute_unprepared(
            "ALTER TABLE audit_logs DROP CONSTRAINT IF EXISTS chk_audit_action",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE audit_logs ADD CONSTRAINT chk_audit_action \
             CHECK (action IN ('CREATE', 'UPDATE', 'ANNUL', 'DELETE', 'VIEW', 'EXPORT', 'DECLARE'))",
        )
        .await?;

        // Restaurar numbering_sequences constraint original
        db.execute_unprepared(
            "ALTER TABLE numbering_sequences DROP CONSTRAINT IF EXISTS chk_sequence_type",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE numbering_sequences ADD CONSTRAINT chk_sequence_type \
             CHECK (sequence_type IN ('INVOICE', 'CREDIT_NOTE', 'DEBIT_NOTE', 'DELIVERY_NOTE'))",
        )
        .await?;

        // Eliminar secuencias ARC insertadas
        db.execute_unprepared(
            "DELETE FROM numbering_sequences WHERE sequence_type = 'ARC'",
        )
        .await?;

        Ok(())
    }
}
