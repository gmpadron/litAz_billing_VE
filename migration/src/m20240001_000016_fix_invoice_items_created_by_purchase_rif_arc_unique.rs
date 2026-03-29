use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ---- Fix 1: Add `created_by` column to `invoice_items` ----
        // All fiscal tables require a `created_by` UUID per project rules.
        // Default value is the nil UUID to handle any existing rows safely.
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE invoice_items \
                 ADD COLUMN created_by UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'",
            )
            .await?;

        // ---- Fix 2: Add CHECK constraint for RIF format on `purchase_book_entries.supplier_rif` ----
        // Mirrors the pattern already used for `sales_book_entries.client_rif`.
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE purchase_book_entries \
                 ADD CONSTRAINT chk_purchase_book_supplier_rif_format \
                 CHECK (supplier_rif ~ '^[JVEGPC]-[0-9]{8}-[0-9]$')",
            )
            .await?;

        // ---- Fix 3: Add named UNIQUE index on `tax_withholdings_islr.arc_number` ----
        // The column was created with `.unique_key()` (an anonymous unique constraint).
        // This adds a named index for explicit, auditable enforcement.
        manager
            .create_index(
                Index::create()
                    .name("idx_tax_withholdings_islr_arc_number_unique")
                    .table(TaxWithholdingsIslr::Table)
                    .col(TaxWithholdingsIslr::ArcNumber)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop Fix 3: named unique index on arc_number
        manager
            .drop_index(
                Index::drop()
                    .name("idx_tax_withholdings_islr_arc_number_unique")
                    .table(TaxWithholdingsIslr::Table)
                    .to_owned(),
            )
            .await?;

        // Drop Fix 2: CHECK constraint on supplier_rif
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE purchase_book_entries \
                 DROP CONSTRAINT chk_purchase_book_supplier_rif_format",
            )
            .await?;

        // Drop Fix 1: created_by column from invoice_items
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE invoice_items DROP COLUMN created_by")
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum TaxWithholdingsIslr {
    Table,
    ArcNumber,
}
