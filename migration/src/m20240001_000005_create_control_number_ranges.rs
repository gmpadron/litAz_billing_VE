use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ControlNumberRanges::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ControlNumberRanges::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ControlNumberRanges::CompanyProfileId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        // Prefix part of the control number e.g. "00"
                        ColumnDef::new(ControlNumberRanges::Prefix)
                            .string_len(2)
                            .not_null(),
                    )
                    .col(
                        // Start of the range (numeric portion)
                        ColumnDef::new(ControlNumberRanges::RangeFrom)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        // End of the range (inclusive)
                        ColumnDef::new(ControlNumberRanges::RangeTo)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        // Current pointer within the range (last used)
                        ColumnDef::new(ControlNumberRanges::CurrentValue)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        // Only one range can be active per company at a time
                        ColumnDef::new(ControlNumberRanges::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        // Name or reference of the authorized printer (imprenta autorizada)
                        ColumnDef::new(ControlNumberRanges::ImprentaAutorizada)
                            .string()
                            .null(),
                    )
                    .col(
                        // Authorization date from imprenta
                        ColumnDef::new(ControlNumberRanges::FechaAutorizacion)
                            .date()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ControlNumberRanges::CreatedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ControlNumberRanges::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ControlNumberRanges::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // CHECK: current_value must be within range
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE control_number_ranges ADD CONSTRAINT chk_control_range_valid \
                 CHECK (range_from <= range_to AND current_value >= range_from AND current_value <= range_to)",
            )
            .await?;

        // Index for company + active lookup
        manager
            .create_index(
                Index::create()
                    .name("idx_control_number_ranges_company_active")
                    .table(ControlNumberRanges::Table)
                    .col(ControlNumberRanges::CompanyProfileId)
                    .col(ControlNumberRanges::IsActive)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_control_number_ranges_created_at")
                    .table(ControlNumberRanges::Table)
                    .col(ControlNumberRanges::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ControlNumberRanges::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ControlNumberRanges {
    Table,
    Id,
    CompanyProfileId,
    Prefix,
    RangeFrom,
    RangeTo,
    CurrentValue,
    IsActive,
    ImprentaAutorizada,
    FechaAutorizacion,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}
