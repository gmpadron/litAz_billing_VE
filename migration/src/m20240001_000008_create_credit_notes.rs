use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CreditNotes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CreditNotes::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CreditNotes::CreditNoteNumber)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(CreditNotes::ControlNumber)
                            .string_len(11)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        // The invoice this credit note corrects
                        ColumnDef::new(CreditNotes::OriginalInvoiceId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CreditNotes::ClientId).uuid().not_null())
                    .col(
                        ColumnDef::new(CreditNotes::CompanyProfileId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CreditNotes::IssueDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        // Reason for the credit note
                        ColumnDef::new(CreditNotes::Reason).text().not_null(),
                    )
                    .col(
                        ColumnDef::new(CreditNotes::Subtotal)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CreditNotes::TaxAmount)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CreditNotes::Total)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CreditNotes::Currency)
                            .string_len(3)
                            .not_null()
                            .default("VES"),
                    )
                    .col(ColumnDef::new(CreditNotes::ExchangeRateId).uuid().null())
                    .col(
                        ColumnDef::new(CreditNotes::ExchangeRateSnapshot)
                            .decimal_len(18, 6)
                            .null(),
                    )
                    .col(
                        // CHECK: Emitida, Anulada
                        ColumnDef::new(CreditNotes::Status)
                            .string()
                            .not_null()
                            .default("Emitida"),
                    )
                    .col(ColumnDef::new(CreditNotes::Notes).text().null())
                    .col(ColumnDef::new(CreditNotes::CreatedBy).uuid().not_null())
                    .col(
                        ColumnDef::new(CreditNotes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CreditNotes::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_credit_notes_invoice")
                            .from(CreditNotes::Table, CreditNotes::OriginalInvoiceId)
                            .to(Invoices::Table, Invoices::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_credit_notes_client")
                            .from(CreditNotes::Table, CreditNotes::ClientId)
                            .to(Clients::Table, Clients::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_credit_notes_company_profile")
                            .from(CreditNotes::Table, CreditNotes::CompanyProfileId)
                            .to(CompanyProfiles::Table, CompanyProfiles::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE credit_notes ADD CONSTRAINT chk_credit_note_status \
                 CHECK (status IN ('Emitida', 'Anulada'))",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE credit_notes ADD CONSTRAINT chk_credit_note_control_number_format \
                 CHECK (control_number ~ '^[0-9]{2}-[0-9]{1,8}$')",
            )
            .await?;

        // Indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_credit_notes_number")
                    .table(CreditNotes::Table)
                    .col(CreditNotes::CreditNoteNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_credit_notes_control_number")
                    .table(CreditNotes::Table)
                    .col(CreditNotes::ControlNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_credit_notes_original_invoice_id")
                    .table(CreditNotes::Table)
                    .col(CreditNotes::OriginalInvoiceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_credit_notes_created_at")
                    .table(CreditNotes::Table)
                    .col(CreditNotes::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CreditNotes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum CreditNotes {
    Table,
    Id,
    CreditNoteNumber,
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
