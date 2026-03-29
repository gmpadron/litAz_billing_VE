use sea_orm::{Database, DatabaseConnection, DbErr};

/// Establece conexión con PostgreSQL usando la URL proporcionada.
pub async fn establish_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(database_url).await
}
