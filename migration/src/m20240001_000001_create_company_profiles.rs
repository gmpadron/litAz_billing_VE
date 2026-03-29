use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CompanyProfiles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CompanyProfiles::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CompanyProfiles::RazonSocial)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CompanyProfiles::NombreComercial)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(CompanyProfiles::Rif)
                            .string_len(12)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(CompanyProfiles::DomicilioFiscal)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CompanyProfiles::Telefono)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CompanyProfiles::Email).string().null())
                    .col(
                        ColumnDef::new(CompanyProfiles::EsContribuyenteEspecial)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(CompanyProfiles::NroContribuyenteEspecial)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(CompanyProfiles::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(CompanyProfiles::CreatedBy).uuid().not_null())
                    .col(
                        ColumnDef::new(CompanyProfiles::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CompanyProfiles::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // CHECK constraint on RIF format [JVEGPC]-XXXXXXXX-X
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE company_profiles ADD CONSTRAINT chk_company_rif_format \
                 CHECK (rif ~ '^[JVEGPC]-[0-9]{8}-[0-9]$')",
            )
            .await?;

        // Index on rif
        manager
            .create_index(
                Index::create()
                    .name("idx_company_profiles_rif")
                    .table(CompanyProfiles::Table)
                    .col(CompanyProfiles::Rif)
                    .to_owned(),
            )
            .await?;

        // Index on created_at
        manager
            .create_index(
                Index::create()
                    .name("idx_company_profiles_created_at")
                    .table(CompanyProfiles::Table)
                    .col(CompanyProfiles::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CompanyProfiles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum CompanyProfiles {
    Table,
    Id,
    RazonSocial,
    NombreComercial,
    Rif,
    DomicilioFiscal,
    Telefono,
    Email,
    EsContribuyenteEspecial,
    NroContribuyenteEspecial,
    IsActive,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}
