use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TaxWithholdingsIslr::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaxWithholdingsIslr::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        // ARC (Comprobante de Retención) number
                        ColumnDef::new(TaxWithholdingsIslr::ArcNumber)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(TaxWithholdingsIslr::InvoiceId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaxWithholdingsIslr::SupplierId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaxWithholdingsIslr::CompanyProfileId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaxWithholdingsIslr::WithholdingDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        // Year-month for monthly declaration e.g. "2026-03"
                        ColumnDef::new(TaxWithholdingsIslr::ReportingPeriod)
                            .string_len(7)
                            .not_null(),
                    )
                    .col(
                        // Activity code per Decreto 1.808
                        ColumnDef::new(TaxWithholdingsIslr::ActivityCode)
                            .string()
                            .not_null(),
                    )
                    .col(
                        // Description of the activity
                        ColumnDef::new(TaxWithholdingsIslr::ActivityDescription)
                            .string()
                            .not_null(),
                    )
                    .col(
                        // Gross amount subject to withholding
                        ColumnDef::new(TaxWithholdingsIslr::TaxableAmount)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        // Withholding rate (%) e.g. 1.00, 2.00, 3.00, 5.00, 34.00
                        ColumnDef::new(TaxWithholdingsIslr::WithholdingRate)
                            .decimal_len(5, 2)
                            .not_null(),
                    )
                    .col(
                        // Withheld amount = taxable_amount * (rate / 100)
                        ColumnDef::new(TaxWithholdingsIslr::WithheldAmount)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        // CHECK: Emitido, Declarado, Anulado
                        ColumnDef::new(TaxWithholdingsIslr::Status)
                            .string()
                            .not_null()
                            .default("Emitido"),
                    )
                    .col(
                        // Path or reference to generated TXT file for SENIAT
                        ColumnDef::new(TaxWithholdingsIslr::TxtFilePath)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TaxWithholdingsIslr::CreatedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaxWithholdingsIslr::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaxWithholdingsIslr::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_withholdings_islr_invoice")
                            .from(TaxWithholdingsIslr::Table, TaxWithholdingsIslr::InvoiceId)
                            .to(Invoices::Table, Invoices::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_withholdings_islr_supplier")
                            .from(TaxWithholdingsIslr::Table, TaxWithholdingsIslr::SupplierId)
                            .to(Clients::Table, Clients::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_withholdings_islr_company")
                            .from(
                                TaxWithholdingsIslr::Table,
                                TaxWithholdingsIslr::CompanyProfileId,
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
                "ALTER TABLE tax_withholdings_islr ADD CONSTRAINT chk_islr_withholding_status \
                 CHECK (status IN ('Emitido', 'Declarado', 'Anulado'))",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE tax_withholdings_islr ADD CONSTRAINT chk_islr_withholding_rate_positive \
                 CHECK (withholding_rate > 0 AND withholding_rate <= 100)",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE tax_withholdings_islr ADD CONSTRAINT chk_islr_reporting_period_format \
                 CHECK (reporting_period ~ '^[0-9]{4}-(0[1-9]|1[0-2])$')",
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tax_withholdings_islr_invoice_id")
                    .table(TaxWithholdingsIslr::Table)
                    .col(TaxWithholdingsIslr::InvoiceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tax_withholdings_islr_reporting_period")
                    .table(TaxWithholdingsIslr::Table)
                    .col(TaxWithholdingsIslr::ReportingPeriod)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tax_withholdings_islr_created_at")
                    .table(TaxWithholdingsIslr::Table)
                    .col(TaxWithholdingsIslr::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TaxWithholdingsIslr::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TaxWithholdingsIslr {
    Table,
    Id,
    ArcNumber,
    InvoiceId,
    SupplierId,
    CompanyProfileId,
    WithholdingDate,
    ReportingPeriod,
    ActivityCode,
    ActivityDescription,
    TaxableAmount,
    WithholdingRate,
    WithheldAmount,
    Status,
    TxtFilePath,
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
