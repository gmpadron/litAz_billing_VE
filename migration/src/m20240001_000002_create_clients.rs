use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Clients::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Clients::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Clients::RazonSocial).string().not_null())
                    .col(ColumnDef::new(Clients::NombreComercial).string().null())
                    .col(
                        // RIF is nullable because consumer final (persona natural sin RIF) can exist
                        ColumnDef::new(Clients::Rif).string_len(12).null(),
                    )
                    .col(ColumnDef::new(Clients::Cedula).string_len(20).null())
                    .col(ColumnDef::new(Clients::DomicilioFiscal).text().null())
                    .col(ColumnDef::new(Clients::Telefono).string().null())
                    .col(ColumnDef::new(Clients::Email).string().null())
                    .col(
                        // true = consumidor final sin RIF (triggers "SIN DERECHO A CRÉDITO FISCAL")
                        ColumnDef::new(Clients::EsConsumidorFinal)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Clients::EsContribuyenteEspecial)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Clients::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(Clients::CreatedBy).uuid().not_null())
                    .col(
                        ColumnDef::new(Clients::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Clients::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // CHECK constraint on RIF format [JVEGPC]-XXXXXXXX-X (only when not null)
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE clients ADD CONSTRAINT chk_client_rif_format \
                 CHECK (rif IS NULL OR rif ~ '^[JVEGPC]-[0-9]{8}-[0-9]$')",
            )
            .await?;

        // CHECK: either rif or cedula or es_consumidor_final must be set
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE clients ADD CONSTRAINT chk_client_identification \
                 CHECK (rif IS NOT NULL OR cedula IS NOT NULL OR es_consumidor_final = TRUE)",
            )
            .await?;

        // Index on rif
        manager
            .create_index(
                Index::create()
                    .name("idx_clients_rif")
                    .table(Clients::Table)
                    .col(Clients::Rif)
                    .to_owned(),
            )
            .await?;

        // Index on created_at
        manager
            .create_index(
                Index::create()
                    .name("idx_clients_created_at")
                    .table(Clients::Table)
                    .col(Clients::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Clients::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Clients {
    Table,
    Id,
    RazonSocial,
    NombreComercial,
    Rif,
    Cedula,
    DomicilioFiscal,
    Telefono,
    Email,
    EsConsumidorFinal,
    EsContribuyenteEspecial,
    IsActive,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}
