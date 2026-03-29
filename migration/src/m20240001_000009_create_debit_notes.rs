use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DebitNotes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DebitNotes::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DebitNotes::DebitNoteNumber)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(DebitNotes::ControlNumber)
                            .string_len(11)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        // The invoice this debit note augments
                        ColumnDef::new(DebitNotes::OriginalInvoiceId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DebitNotes::ClientId).uuid().not_null())
                    .col(
                        ColumnDef::new(DebitNotes::CompanyProfileId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DebitNotes::IssueDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DebitNotes::Reason).text().not_null())
                    .col(
                        ColumnDef::new(DebitNotes::Subtotal)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DebitNotes::TaxAmount)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DebitNotes::Total)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DebitNotes::Currency)
                            .string_len(3)
                            .not_null()
                            .default("VES"),
                    )
                    .col(ColumnDef::new(DebitNotes::ExchangeRateId).uuid().null())
                    .col(
                        ColumnDef::new(DebitNotes::ExchangeRateSnapshot)
                            .decimal_len(18, 6)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DebitNotes::Status)
                            .string()
                            .not_null()
                            .default("Emitida"),
                    )
                    .col(ColumnDef::new(DebitNotes::Notes).text().null())
                    .col(ColumnDef::new(DebitNotes::CreatedBy).uuid().not_null())
                    .col(
                        ColumnDef::new(DebitNotes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DebitNotes::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_debit_notes_invoice")
                            .from(DebitNotes::Table, DebitNotes::OriginalInvoiceId)
                            .to(Invoices::Table, Invoices::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_debit_notes_client")
                            .from(DebitNotes::Table, DebitNotes::ClientId)
                            .to(Clients::Table, Clients::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_debit_notes_company_profile")
                            .from(DebitNotes::Table, DebitNotes::CompanyProfileId)
                            .to(CompanyProfiles::Table, CompanyProfiles::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE debit_notes ADD CONSTRAINT chk_debit_note_status \
                 CHECK (status IN ('Emitida', 'Anulada'))",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE debit_notes ADD CONSTRAINT chk_debit_note_control_number_format \
                 CHECK (control_number ~ '^[0-9]{2}-[0-9]{1,8}$')",
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_debit_notes_number")
                    .table(DebitNotes::Table)
                    .col(DebitNotes::DebitNoteNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_debit_notes_control_number")
                    .table(DebitNotes::Table)
                    .col(DebitNotes::ControlNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_debit_notes_original_invoice_id")
                    .table(DebitNotes::Table)
                    .col(DebitNotes::OriginalInvoiceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_debit_notes_created_at")
                    .table(DebitNotes::Table)
                    .col(DebitNotes::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DebitNotes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum DebitNotes {
    Table,
    Id,
    DebitNoteNumber,
    ControlNumber,
    OriginalInvoiceId,
    ClientId,
    CompanyProfileId,
    IssueDate,
    Reason,
    Subtotal,
    TaxAmount,
    Total,
    Currency,
    ExchangeRateId,
    ExchangeRateSnapshot,
    Status,
    Notes,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Invoices {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Clients {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum CompanyProfiles {
    Table,
    Id,
}
