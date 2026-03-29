use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditLogs::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        // UUID from JWT (who performed the action)
                        ColumnDef::new(AuditLogs::UserId).uuid().not_null(),
                    )
                    .col(
                        // Action performed: CREATE, UPDATE, ANNUL, etc.
                        ColumnDef::new(AuditLogs::Action).string().not_null(),
                    )
                    .col(
                        // Type of entity: invoice, credit_note, debit_note, etc.
                        ColumnDef::new(AuditLogs::EntityType).string().not_null(),
                    )
                    .col(
                        // UUID of the affected entity
                        ColumnDef::new(AuditLogs::EntityId).uuid().not_null(),
                    )
                    .col(
                        // JSONB diff of what changed (before/after)
                        ColumnDef::new(AuditLogs::Changes).json_binary().null(),
                    )
                    .col(
                        // Additional context (IP address, user agent, etc.)
                        ColumnDef::new(AuditLogs::Metadata).json_binary().null(),
                    )
                    .col(
                        ColumnDef::new(AuditLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    // audit_logs intentionally has NO updated_at (immutable records)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE audit_logs ADD CONSTRAINT chk_audit_action \
                 CHECK (action IN ('CREATE', 'UPDATE', 'ANNUL', 'DELETE', 'VIEW', 'EXPORT', 'DECLARE'))",
            )
            .await?;

        // Indexes for common query patterns
        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_user_id")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_entity_type_id")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::EntityType)
                    .col(AuditLogs::EntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_created_at")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_action")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::Action)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum AuditLogs {
    Table,
    Id,
    UserId,
    Action,
    EntityType,
    EntityId,
    Changes,
    Metadata,
    CreatedAt,
}
