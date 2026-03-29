use sea_orm_migration::prelude::*;

/// Adds optional FK constraints on:
/// - `purchase_book_entries.invoice_id` → `invoices.id` ON DELETE SET NULL
/// - `sales_book_entries.invoice_id`   → `invoices.id` ON DELETE SET NULL
///
/// The `invoice_id` columns already exist as nullable UUIDs; this migration
/// adds the referential integrity that was missing.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ---- FK: purchase_book_entries.invoice_id → invoices.id ----
        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseBookEntries::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_purchase_book_invoice_id")
                            .from_tbl(PurchaseBookEntries::Table)
                            .from_col(PurchaseBookEntries::InvoiceId)
                            .to_tbl(Invoices::Table)
                            .to_col(Invoices::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // ---- FK: sales_book_entries.invoice_id → invoices.id ----
        manager
            .alter_table(
                Table::alter()
                    .table(SalesBookEntries::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_sales_book_invoice_id")
                            .from_tbl(SalesBookEntries::Table)
                            .from_col(SalesBookEntries::InvoiceId)
                            .to_tbl(Invoices::Table)
                            .to_col(Invoices::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop FK from sales_book_entries first (reverse order)
        manager
            .alter_table(
                Table::alter()
                    .table(SalesBookEntries::Table)
                    .drop_foreign_key(Alias::new("fk_sales_book_invoice_id"))
                    .to_owned(),
            )
            .await?;

        // Drop FK from purchase_book_entries
        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseBookEntries::Table)
                    .drop_foreign_key(Alias::new("fk_purchase_book_invoice_id"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Invoices {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum PurchaseBookEntries {
    Table,
    InvoiceId,
}

#[derive(DeriveIden)]
enum SalesBookEntries {
    Table,
    InvoiceId,
}
