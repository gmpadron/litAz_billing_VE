use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InvoiceItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InvoiceItems::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(InvoiceItems::InvoiceId).uuid().not_null())
                    .col(
                        // Line number within the invoice (for ordering)
                        ColumnDef::new(InvoiceItems::LineNumber)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InvoiceItems::Description).text().not_null())
                    .col(
                        ColumnDef::new(InvoiceItems::Quantity)
                            .decimal_len(18, 4)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InvoiceItems::UnitPrice)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        // Discount amount (if any)
                        ColumnDef::new(InvoiceItems::Discount)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        // Net subtotal after discount = (quantity * unit_price) - discount
                        ColumnDef::new(InvoiceItems::Subtotal)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        // IVA rate as percentage e.g. 16.00, 8.00, 0.00, 31.00
                        ColumnDef::new(InvoiceItems::TaxRate)
                            .decimal_len(5, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        // IVA type: GENERAL, REDUCIDA, LUJO, EXENTO
                        ColumnDef::new(InvoiceItems::TaxType)
                            .string()
                            .not_null()
                            .default("EXENTO"),
                    )
                    .col(
                        ColumnDef::new(InvoiceItems::TaxAmount)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        // Total for this line = subtotal + tax_amount
                        ColumnDef::new(InvoiceItems::Total)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        // Optional product/service code
                        ColumnDef::new(InvoiceItems::ProductCode).string().null(),
                    )
                    .col(
                        ColumnDef::new(InvoiceItems::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InvoiceItems::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invoice_items_invoice")
                            .from(InvoiceItems::Table, InvoiceItems::InvoiceId)
                            .to(Invoices::Table, Invoices::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // CHECK constraints
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE invoice_items ADD CONSTRAINT chk_item_tax_type \
                 CHECK (tax_type IN ('GENERAL', 'REDUCIDA', 'LUJO', 'EXENTO'))",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE invoice_items ADD CONSTRAINT chk_item_quantity_positive \
                 CHECK (quantity > 0)",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE invoice_items ADD CONSTRAINT chk_item_unit_price_positive \
                 CHECK (unit_price >= 0)",
            )
            .await?;

        // Indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_invoice_items_invoice_id")
                    .table(InvoiceItems::Table)
                    .col(InvoiceItems::InvoiceId)
                    .to_owned(),
            )
            .await?;

        // Unique line number per invoice
        manager
            .create_index(
                Index::create()
                    .name("idx_invoice_items_invoice_line_unique")
                    .table(InvoiceItems::Table)
                    .col(InvoiceItems::InvoiceId)
                    .col(InvoiceItems::LineNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InvoiceItems::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum InvoiceItems {
    Table,
    Id,
    InvoiceId,
    LineNumber,
    Description,
    Quantity,
    UnitPrice,
    Discount,
    Subtotal,
    TaxRate,
    TaxType,
    TaxAmount,
    Total,
    ProductCode,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Invoices {
    Table,
    Id,
}
