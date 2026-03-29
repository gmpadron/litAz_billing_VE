use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DeliveryNotes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DeliveryNotes::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DeliveryNotes::DeliveryNoteNumber)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(DeliveryNotes::ControlNumber)
                            .string_len(11)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        // Related invoice (optional — delivery note can precede invoice)
                        ColumnDef::new(DeliveryNotes::InvoiceId).uuid().null(),
                    )
                    .col(ColumnDef::new(DeliveryNotes::ClientId).uuid().not_null())
                    .col(
                        ColumnDef::new(DeliveryNotes::CompanyProfileId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeliveryNotes::IssueDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        // Delivery address (can differ from fiscal address)
                        ColumnDef::new(DeliveryNotes::DeliveryAddress).text().null(),
                    )
                    .col(ColumnDef::new(DeliveryNotes::Notes).text().null())
                    .col(
                        // CHECK: Emitida, Anulada, Entregada
                        ColumnDef::new(DeliveryNotes::Status)
                            .string()
                            .not_null()
                            .default("Emitida"),
                    )
                    .col(ColumnDef::new(DeliveryNotes::CreatedBy).uuid().not_null())
                    .col(
                        ColumnDef::new(DeliveryNotes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeliveryNotes::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_delivery_notes_invoice")
                            .from(DeliveryNotes::Table, DeliveryNotes::InvoiceId)
                            .to(Invoices::Table, Invoices::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_delivery_notes_client")
                            .from(DeliveryNotes::Table, DeliveryNotes::ClientId)
                            .to(Clients::Table, Clients::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_delivery_notes_company_profile")
                            .from(DeliveryNotes::Table, DeliveryNotes::CompanyProfileId)
                            .to(CompanyProfiles::Table, CompanyProfiles::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE delivery_notes ADD CONSTRAINT chk_delivery_note_status \
                 CHECK (status IN ('Emitida', 'Anulada', 'Entregada'))",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE delivery_notes ADD CONSTRAINT chk_delivery_note_control_number_format \
                 CHECK (control_number ~ '^[0-9]{2}-[0-9]{1,8}$')",
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_delivery_notes_number")
                    .table(DeliveryNotes::Table)
                    .col(DeliveryNotes::DeliveryNoteNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_delivery_notes_control_number")
                    .table(DeliveryNotes::Table)
                    .col(DeliveryNotes::ControlNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_delivery_notes_invoice_id")
                    .table(DeliveryNotes::Table)
                    .col(DeliveryNotes::InvoiceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_delivery_notes_created_at")
                    .table(DeliveryNotes::Table)
                    .col(DeliveryNotes::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DeliveryNotes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum DeliveryNotes {
    Table,
    Id,
    DeliveryNoteNumber,
    ControlNumber,
    InvoiceId,
    ClientId,
    CompanyProfileId,
    IssueDate,
    DeliveryAddress,
    Notes,
    Status,
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
