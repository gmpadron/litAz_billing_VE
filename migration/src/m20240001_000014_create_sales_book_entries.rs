use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SalesBookEntries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SalesBookEntries::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::CompanyProfileId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        // Book period: "2026-03" (year-month)
                        ColumnDef::new(SalesBookEntries::Period)
                            .string_len(7)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::EntryDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    // ---- Client info (snapshot) ----
                    .col(
                        // Null for consumer final (daily summary entries)
                        ColumnDef::new(SalesBookEntries::ClientRif)
                            .string_len(12)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::ClientName)
                            .string()
                            .not_null(),
                    )
                    // ---- Document info ----
                    .col(
                        ColumnDef::new(SalesBookEntries::InvoiceNumber)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::ControlNumber)
                            .string_len(11)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::InvoiceDate)
                            .date()
                            .not_null(),
                    )
                    // ---- Amounts ----
                    .col(
                        ColumnDef::new(SalesBookEntries::TotalAmount)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::BaseImponibleExenta)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::BaseImponibleReducida)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::BaseImponibleGeneral)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::BaseImponibleLujo)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::IvaReducida)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::IvaGeneral)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::IvaLujo)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    // ---- Consumer final daily summary ----
                    .col(
                        // True if this is a resumen diario (grouped consumer final sales)
                        ColumnDef::new(SalesBookEntries::EsResumenDiario)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // ---- Reference to original document ----
                    .col(ColumnDef::new(SalesBookEntries::InvoiceId).uuid().null())
                    .col(
                        ColumnDef::new(SalesBookEntries::CreatedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SalesBookEntries::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sales_book_company")
                            .from(SalesBookEntries::Table, SalesBookEntries::CompanyProfileId)
                            .to(CompanyProfiles::Table, CompanyProfiles::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE sales_book_entries ADD CONSTRAINT chk_sales_book_period_format \
                 CHECK (period ~ '^[0-9]{4}-(0[1-9]|1[0-2])$')",
            )
            .await?;

        // Client RIF validation when present
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE sales_book_entries ADD CONSTRAINT chk_sales_book_client_rif_format \
                 CHECK (client_rif IS NULL OR client_rif ~ '^[JVEGPC]-[0-9]{8}-[0-9]$')",
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sales_book_company_period")
                    .table(SalesBookEntries::Table)
                    .col(SalesBookEntries::CompanyProfileId)
                    .col(SalesBookEntries::Period)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sales_book_client_rif")
                    .table(SalesBookEntries::Table)
                    .col(SalesBookEntries::ClientRif)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sales_book_invoice_number")
                    .table(SalesBookEntries::Table)
                    .col(SalesBookEntries::InvoiceNumber)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sales_book_created_at")
                    .table(SalesBookEntries::Table)
                    .col(SalesBookEntries::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SalesBookEntries::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum SalesBookEntries {
    Table,
    Id,
    CompanyProfileId,
    Period,
    EntryDate,
    ClientRif,
    ClientName,
    InvoiceNumber,
    ControlNumber,
    InvoiceDate,
    TotalAmount,
    BaseImponibleExenta,
    BaseImponibleReducida,
    BaseImponibleGeneral,
    BaseImponibleLujo,
    IvaReducida,
    IvaGeneral,
    IvaLujo,
    EsResumenDiario,
    InvoiceId,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum CompanyProfiles {
    Table,
    Id,
}
