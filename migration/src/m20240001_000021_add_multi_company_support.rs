use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── 1. clients: agregar company_profile_id ───────────────────────────

        manager
            .alter_table(
                Table::alter()
                    .table(Clients::Table)
                    .add_column(ColumnDef::new(Clients::CompanyProfileId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Backfill con la empresa activa existente
        db.execute_unprepared(
            "UPDATE clients \
             SET company_profile_id = (SELECT id FROM company_profiles WHERE is_active = TRUE LIMIT 1) \
             WHERE company_profile_id IS NULL",
        )
        .await?;

        // Convertir a NOT NULL
        db.execute_unprepared(
            "ALTER TABLE clients ALTER COLUMN company_profile_id SET NOT NULL",
        )
        .await?;

        // FK clients -> company_profiles
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_clients_company_profile")
                    .from(Clients::Table, Clients::CompanyProfileId)
                    .to(CompanyProfiles::Table, CompanyProfiles::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // Eliminar unique global en RIF (si existe)
        db.execute_unprepared(
            "ALTER TABLE clients DROP CONSTRAINT IF EXISTS clients_rif_key",
        )
        .await?;

        // Nuevo unique compuesto (rif, company_profile_id) — solo cuando rif no es null
        db.execute_unprepared(
            "CREATE UNIQUE INDEX idx_clients_rif_company \
             ON clients(rif, company_profile_id) \
             WHERE rif IS NOT NULL",
        )
        .await?;

        // Índice de búsqueda por empresa
        manager
            .create_index(
                Index::create()
                    .name("idx_clients_company_profile_id")
                    .table(Clients::Table)
                    .col(Clients::CompanyProfileId)
                    .to_owned(),
            )
            .await?;

        // ── 2. audit_logs: agregar company_profile_id (nullable) ─────────────

        manager
            .alter_table(
                Table::alter()
                    .table(AuditLogs::Table)
                    .add_column(
                        ColumnDef::new(AuditLogs::CompanyProfileId).uuid().null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_audit_logs_company_profile")
                    .from(AuditLogs::Table, AuditLogs::CompanyProfileId)
                    .to(CompanyProfiles::Table, CompanyProfiles::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        db.execute_unprepared(
            "CREATE INDEX idx_audit_logs_company_profile_id \
             ON audit_logs(company_profile_id) \
             WHERE company_profile_id IS NOT NULL",
        )
        .await?;

        // ── 3. Índices de rendimiento en tablas que ya tienen company_profile_id

        for sql in [
            "CREATE INDEX IF NOT EXISTS idx_invoices_company_profile_id ON invoices(company_profile_id)",
            "CREATE INDEX IF NOT EXISTS idx_credit_notes_company_profile_id ON credit_notes(company_profile_id)",
            "CREATE INDEX IF NOT EXISTS idx_debit_notes_company_profile_id ON debit_notes(company_profile_id)",
            "CREATE INDEX IF NOT EXISTS idx_delivery_notes_company_profile_id ON delivery_notes(company_profile_id)",
            "CREATE INDEX IF NOT EXISTS idx_twh_iva_company_id ON tax_withholdings_iva(company_profile_id)",
            "CREATE INDEX IF NOT EXISTS idx_twh_islr_company_id ON tax_withholdings_islr(company_profile_id)",
            "CREATE INDEX IF NOT EXISTS idx_pbe_company_id ON purchase_book_entries(company_profile_id)",
            "CREATE INDEX IF NOT EXISTS idx_sbe_company_id ON sales_book_entries(company_profile_id)",
        ] {
            db.execute_unprepared(sql).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Eliminar índices extra
        for sql in [
            "DROP INDEX IF EXISTS idx_sbe_company_id",
            "DROP INDEX IF EXISTS idx_pbe_company_id",
            "DROP INDEX IF EXISTS idx_twh_islr_company_id",
            "DROP INDEX IF EXISTS idx_twh_iva_company_id",
            "DROP INDEX IF EXISTS idx_delivery_notes_company_profile_id",
            "DROP INDEX IF EXISTS idx_debit_notes_company_profile_id",
            "DROP INDEX IF EXISTS idx_credit_notes_company_profile_id",
            "DROP INDEX IF EXISTS idx_invoices_company_profile_id",
            "DROP INDEX IF EXISTS idx_audit_logs_company_profile_id",
        ] {
            db.execute_unprepared(sql).await?;
        }

        // audit_logs: revertir
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_audit_logs_company_profile")
                    .table(AuditLogs::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AuditLogs::Table)
                    .drop_column(AuditLogs::CompanyProfileId)
                    .to_owned(),
            )
            .await?;

        // clients: revertir
        db.execute_unprepared("DROP INDEX IF EXISTS idx_clients_rif_company").await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_clients_company_profile_id")
                    .table(Clients::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_clients_company_profile")
                    .table(Clients::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Clients::Table)
                    .drop_column(Clients::CompanyProfileId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Clients {
    Table,
    CompanyProfileId,
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    CompanyProfileId,
}

#[derive(DeriveIden)]
enum CompanyProfiles {
    Table,
    Id,
}
