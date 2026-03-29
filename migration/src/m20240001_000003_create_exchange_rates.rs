use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExchangeRates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExchangeRates::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        // The date this rate applies to (not datetime, just date)
                        ColumnDef::new(ExchangeRates::RateDate).date().not_null(),
                    )
                    .col(
                        // Base currency (always VES/Bs)
                        ColumnDef::new(ExchangeRates::BaseCurrency)
                            .string_len(3)
                            .not_null()
                            .default("VES"),
                    )
                    .col(
                        // Quote currency (usually USD)
                        ColumnDef::new(ExchangeRates::QuoteCurrency)
                            .string_len(3)
                            .not_null()
                            .default("USD"),
                    )
                    .col(
                        // How many Bs per 1 USD — DECIMAL(18,6) for precision
                        ColumnDef::new(ExchangeRates::Rate)
                            .decimal_len(18, 6)
                            .not_null(),
                    )
                    .col(
                        // Source: BCV, PARALLEL, MANUAL, etc.
                        ColumnDef::new(ExchangeRates::Source)
                            .string()
                            .not_null()
                            .default("BCV"),
                    )
                    .col(ColumnDef::new(ExchangeRates::CreatedBy).uuid().not_null())
                    .col(
                        ColumnDef::new(ExchangeRates::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExchangeRates::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint: one rate per currency pair per date
        manager
            .create_index(
                Index::create()
                    .name("idx_exchange_rates_date_currencies_unique")
                    .table(ExchangeRates::Table)
                    .col(ExchangeRates::RateDate)
                    .col(ExchangeRates::BaseCurrency)
                    .col(ExchangeRates::QuoteCurrency)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index on rate_date for quick lookups by date
        manager
            .create_index(
                Index::create()
                    .name("idx_exchange_rates_created_at")
                    .table(ExchangeRates::Table)
                    .col(ExchangeRates::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExchangeRates::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ExchangeRates {
    Table,
    Id,
    RateDate,
    BaseCurrency,
    QuoteCurrency,
    Rate,
    Source,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}
