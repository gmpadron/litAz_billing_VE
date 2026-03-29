use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PurchaseBookEntries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PurchaseBookEntries::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::CompanyProfileId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        // Book period: "2026-03" (year-month)
                        ColumnDef::new(PurchaseBookEntries::Period)
                            .string_len(7)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::EntryDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    // ---- Supplier info (snapshot at time of entry) ----
                    .col(
                        ColumnDef::new(PurchaseBookEntries::SupplierRif)
                            .string_len(12)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::SupplierName)
                            .string()
                            .not_null(),
                    )
                    // ---- Document info ----
                    .col(
                        ColumnDef::new(PurchaseBookEntries::InvoiceNumber)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::ControlNumber)
                            .string_len(11)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::InvoiceDate)
                            .date()
                            .not_null(),
                    )
                    // ---- Amounts ----
                    .col(
                        ColumnDef::new(PurchaseBookEntries::TotalAmount)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::BaseImponibleExenta)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::BaseImponibleReducida)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::BaseImponibleGeneral)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::BaseImponibleLujo)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::IvaReducida)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::IvaGeneral)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::IvaLujo)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        // IVA retained (if applicable)
                        ColumnDef::new(PurchaseBookEntries::IvaRetenido)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    // ---- Type of operation ----
                    .col(
                        // COMPRA, IMPORTACION, SERVICIO, etc.
                        ColumnDef::new(PurchaseBookEntries::OperationType)
                            .string()
                            .not_null()
                            .default("COMPRA"),
                    )
                    // ---- Reference to original document ----
                    .col(ColumnDef::new(PurchaseBookEntries::InvoiceId).uuid().null())
                    .col(
                        ColumnDef::new(PurchaseBookEntries::CreatedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PurchaseBookEntries::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_purchase_book_company")
                            .from(
                                PurchaseBookEntries::Table,
                                PurchaseBookEntries::CompanyProfileId,
                            )
                            .to(CompanyProfiles::Table, CompanyProfiles::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE purchase_book_entries ADD CONSTRAINT chk_purchase_book_period_format \
                 CHECK (period ~ '^[0-9]{4}-(0[1-9]|1[0-2])$')",
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_purchase_book_company_period")
                    .table(PurchaseBookEntries::Table)
                    .col(PurchaseBookEntries::CompanyProfileId)
                    .col(PurchaseBookEntries::Period)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_purchase_book_supplier_rif")
                    .table(PurchaseBookEntries::Table)
                    .col(PurchaseBookEntries::SupplierRif)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_purchase_book_invoice_number")
                    .table(PurchaseBookEntries::Table)
                    .col(PurchaseBookEntries::InvoiceNumber)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_purchase_book_created_at")
                    .table(PurchaseBookEntries::Table)
                    .col(PurchaseBookEntries::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PurchaseBookEntries::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PurchaseBookEntries {
    Table,
    Id,
    CompanyProfileId,
    Period,
    EntryDate,
    SupplierRif,
    SupplierName,
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
    IvaRetenido,
    OperationType,
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
