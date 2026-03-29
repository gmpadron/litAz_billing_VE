use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Invoices::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Invoices::Id).uuid().not_null().primary_key())
                    // ---- Document identifiers ----
                    .col(
                        ColumnDef::new(Invoices::InvoiceNumber)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        // Format: 00-XXXXXXXX
                        ColumnDef::new(Invoices::ControlNumber)
                            .string_len(11)
                            .not_null()
                            .unique_key(),
                    )
                    // ---- Dates ----
                    .col(
                        ColumnDef::new(Invoices::InvoiceDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Invoices::DueDate)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // ---- Parties ----
                    .col(ColumnDef::new(Invoices::ClientId).uuid().not_null())
                    .col(ColumnDef::new(Invoices::CompanyProfileId).uuid().not_null())
                    // ---- Financial amounts — all DECIMAL(18,2) ----
                    .col(
                        ColumnDef::new(Invoices::SubtotalExento)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Invoices::SubtotalReducida)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Invoices::SubtotalGeneral)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Invoices::SubtotalLujo)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        // Sum of all subtotals
                        ColumnDef::new(Invoices::Subtotal)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        // IVA reducida (8%)
                        ColumnDef::new(Invoices::IvaReducida)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        // IVA general (16%)
                        ColumnDef::new(Invoices::IvaGeneral)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        // IVA lujo (up to 31%)
                        ColumnDef::new(Invoices::IvaLujo)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        // Total IVA
                        ColumnDef::new(Invoices::TaxAmount)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    .col(
                        // Grand total = subtotal + tax_amount
                        ColumnDef::new(Invoices::Total)
                            .decimal_len(18, 2)
                            .not_null(),
                    )
                    // ---- Currency ----
                    .col(
                        // ISO currency code: VES, USD
                        ColumnDef::new(Invoices::Currency)
                            .string_len(3)
                            .not_null()
                            .default("VES"),
                    )
                    .col(ColumnDef::new(Invoices::ExchangeRateId).uuid().null())
                    .col(
                        // Snapshot of the exchange rate at invoice time
                        ColumnDef::new(Invoices::ExchangeRateSnapshot)
                            .decimal_len(18, 6)
                            .null(),
                    )
                    // ---- Status and payment ----
                    .col(
                        // CHECK: Emitida, Anulada
                        ColumnDef::new(Invoices::Status)
                            .string()
                            .not_null()
                            .default("Emitida"),
                    )
                    .col(
                        // CHECK: Contado, Crédito
                        ColumnDef::new(Invoices::PaymentCondition)
                            .string()
                            .not_null(),
                    )
                    .col(
                        // Days of credit (only for PaymentCondition = Crédito)
                        ColumnDef::new(Invoices::CreditDays).integer().null(),
                    )
                    .col(
                        // "SIN DERECHO A CRÉDITO FISCAL" flag
                        ColumnDef::new(Invoices::NoFiscalCredit)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // ---- Annotations ----
                    .col(ColumnDef::new(Invoices::Notes).text().null())
                    .col(
                        // Reason for annulment if status = Anulada
                        ColumnDef::new(Invoices::AnnulmentReason).text().null(),
                    )
                    // ---- Audit ----
                    .col(ColumnDef::new(Invoices::CreatedBy).uuid().not_null())
                    .col(
                        ColumnDef::new(Invoices::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Invoices::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    // ---- Foreign keys ----
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invoices_client")
                            .from(Invoices::Table, Invoices::ClientId)
                            .to(Clients::Table, Clients::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invoices_company_profile")
                            .from(Invoices::Table, Invoices::CompanyProfileId)
                            .to(CompanyProfiles::Table, CompanyProfiles::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invoices_exchange_rate")
                            .from(Invoices::Table, Invoices::ExchangeRateId)
                            .to(ExchangeRates::Table, ExchangeRates::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // CHECK constraints
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE invoices ADD CONSTRAINT chk_invoice_status \
                 CHECK (status IN ('Emitida', 'Anulada'))",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE invoices ADD CONSTRAINT chk_invoice_payment_condition \
                 CHECK (payment_condition IN ('Contado', 'Crédito'))",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE invoices ADD CONSTRAINT chk_invoice_credit_days \
                 CHECK (payment_condition != 'Crédito' OR credit_days IS NOT NULL)",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE invoices ADD CONSTRAINT chk_invoice_control_number_format \
                 CHECK (control_number ~ '^[0-9]{2}-[0-9]{1,8}$')",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE invoices ADD CONSTRAINT chk_invoice_amounts_positive \
                 CHECK (subtotal >= 0 AND tax_amount >= 0 AND total >= 0)",
            )
            .await?;

        // Indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_invoices_invoice_number")
                    .table(Invoices::Table)
                    .col(Invoices::InvoiceNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invoices_control_number")
                    .table(Invoices::Table)
                    .col(Invoices::ControlNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invoices_client_id")
                    .table(Invoices::Table)
                    .col(Invoices::ClientId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invoices_company_profile_id")
                    .table(Invoices::Table)
                    .col(Invoices::CompanyProfileId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invoices_invoice_date")
                    .table(Invoices::Table)
                    .col(Invoices::InvoiceDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invoices_created_at")
                    .table(Invoices::Table)
                    .col(Invoices::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invoices_status")
                    .table(Invoices::Table)
                    .col(Invoices::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Invoices::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Invoices {
    Table,
    Id,
    InvoiceNumber,
    ControlNumber,
    InvoiceDate,
    DueDate,
    ClientId,
    CompanyProfileId,
    SubtotalExento,
    SubtotalReducida,
    SubtotalGeneral,
    SubtotalLujo,
    Subtotal,
    IvaReducida,
    IvaGeneral,
    IvaLujo,
    TaxAmount,
    Total,
    Currency,
    ExchangeRateId,
    ExchangeRateSnapshot,
    Status,
    PaymentCondition,
    CreditDays,
    NoFiscalCredit,
    Notes,
    AnnulmentReason,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

// Referenced tables (for FK definitions)
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

#[derive(DeriveIden)]
enum ExchangeRates {
    Table,
    Id,
}
