use sea_orm_migration::prelude::*;

/// Habilita la extensión `unaccent` de PostgreSQL.
///
/// Necesaria para búsquedas insensibles a acentos en el listado de clientes
/// (y futuros listados): `unaccent(lower(razon_social)) ILIKE unaccent(lower($1))`.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240001_000022_enable_unaccent_extension"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("CREATE EXTENSION IF NOT EXISTS unaccent")
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP EXTENSION IF EXISTS unaccent")
            .await?;
        Ok(())
    }
}
