use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TaxWithholdingsIva::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaxWithholdingsIva::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        // Withholding voucher number (comprobante de retención)
                        ColumnDef::new(TaxWithholdingsIva::VoucherNumber)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        // The invoice being withheld
                        ColumnDef::new(TaxWithholdingsIva::InvoiceId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        // The supplier (client in our DB) being withheld from
                        ColumnDef::new(TaxWithholdingsIva::SupplierId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaxWithholdingsIva::CompanyProfileId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        // Date of the withholding
                        ColumnDef::new(TaxWithholdingsIva::WithholdingDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        // Quincenal period: PRIMERA (1-15) or SEGUNDA (16-end)
                        ColumnDef::new(TaxWithholdingsIva::Period)
                            .string()
                            .not_null(),
                    )
                    .col(
                        // Year and month for reporting e.g. "2026-03"
                        ColumnDef::new(TaxWithholdingsIva::ReportingPeriod)
                            .string_len(7)
                            .not_null(),
                    )
                    .col(
                        // IVA amount on the invoice
                        ColumnDef::new(TaxWithholdingsIva::IvaAmount)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        // Withholding percentage: 75 or 100
                        ColumnDef::new(TaxWithholdingsIva::WithholdingPercentage)
                            .decimal_len(5, 2)
                            .not_null(),
                    )
                    .col(
                        // Actual withheld amount = iva_amount * (percentage / 100)
                        ColumnDef::new(TaxWithholdingsIva::WithheldAmount)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        // CHECK: Emitido, Declarado, Anulado
                        ColumnDef::new(TaxWithholdingsIva::Status)
                            .string()
                            .not_null()
                            .default("Emitido"),
                    )
                    .col(
                        // Path or reference to generated XML file for SENIAT
                        ColumnDef::new(TaxWithholdingsIva::XmlFilePath)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TaxWithholdingsIva::CreatedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaxWithholdingsIva::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaxWithholdingsIva::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_withholdings_iva_invoice")
                            .from(TaxWithholdingsIva::Table, TaxWithholdingsIva::InvoiceId)
                            .to(Invoices::Table, Invoices::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_withholdings_iva_supplier")
                            .from(TaxWithholdingsIva::Table, TaxWithholdingsIva::SupplierId)
                            .to(Clients::Table, Clients::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_withholdings_iva_company")
                            .from(
                                TaxWithholdingsIva::Table,
                                TaxWithholdingsIva::CompanyProfileId,
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
                "ALTER TABLE tax_withholdings_iva ADD CONSTRAINT chk_iva_withholding_percentage \
                 CHECK (withholding_percentage IN (75.00, 100.00))",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE tax_withholdings_iva ADD CONSTRAINT chk_iva_withholding_period \
                 CHECK (period IN ('PRIMERA', 'SEGUNDA'))",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE tax_withholdings_iva ADD CONSTRAINT chk_iva_withholding_status \
                 CHECK (status IN ('Emitido', 'Declarado', 'Anulado'))",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE tax_withholdings_iva ADD CONSTRAINT chk_iva_reporting_period_format \
                 CHECK (reporting_period ~ '^[0-9]{4}-(0[1-9]|1[0-2])$')",
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tax_withholdings_iva_invoice_id")
                    .table(TaxWithholdingsIva::Table)
                    .col(TaxWithholdingsIva::InvoiceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tax_withholdings_iva_reporting_period")
                    .table(TaxWithholdingsIva::Table)
                    .col(TaxWithholdingsIva::ReportingPeriod)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tax_withholdings_iva_created_at")
                    .table(TaxWithholdingsIva::Table)
                    .col(TaxWithholdingsIva::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TaxWithholdingsIva::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TaxWithholdingsIva {
    Table,
    Id,
    VoucherNumber,
    InvoiceId,
    SupplierId,
    CompanyProfileId,
    WithholdingDate,
    Period,
    ReportingPeriod,
    IvaAmount,
    WithholdingPercentage,
    WithheldAmount,
    Status,
    XmlFilePath,
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
