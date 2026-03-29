use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NumberingSequences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NumberingSequences::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        // e.g. INVOICE, CREDIT_NOTE, DEBIT_NOTE, DELIVERY_NOTE
                        ColumnDef::new(NumberingSequences::SequenceType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        // Optional prefix e.g. "FAC-", "NC-", "ND-"
                        ColumnDef::new(NumberingSequences::Prefix).string().null(),
                    )
                    .col(
                        // Current value — next number will be current_value + 1
                        ColumnDef::new(NumberingSequences::CurrentValue)
                            .big_integer()
                            .not_null()
                            .default(0i64),
                    )
                    .col(
                        // Minimum value for the sequence
                        ColumnDef::new(NumberingSequences::MinValue)
                            .big_integer()
                            .not_null()
                            .default(1i64),
                    )
                    .col(
                        // Optional max value (null = unlimited)
                        ColumnDef::new(NumberingSequences::MaxValue)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(NumberingSequences::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(NumberingSequences::CompanyProfileId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NumberingSequences::CreatedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NumberingSequences::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NumberingSequences::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique: one active sequence per type per company
        manager
            .create_index(
                Index::create()
                    .name("idx_numbering_sequences_type_company_unique")
                    .table(NumberingSequences::Table)
                    .col(NumberingSequences::SequenceType)
                    .col(NumberingSequences::CompanyProfileId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_numbering_sequences_created_at")
                    .table(NumberingSequences::Table)
                    .col(NumberingSequences::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // CHECK: valid sequence types
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE numbering_sequences ADD CONSTRAINT chk_sequence_type \
                 CHECK (sequence_type IN ('INVOICE', 'CREDIT_NOTE', 'DEBIT_NOTE', 'DELIVERY_NOTE'))",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NumberingSequences::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum NumberingSequences {
    Table,
    Id,
    SequenceType,
    Prefix,
    CurrentValue,
    MinValue,
    MaxValue,
    IsActive,
    CompanyProfileId,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}
