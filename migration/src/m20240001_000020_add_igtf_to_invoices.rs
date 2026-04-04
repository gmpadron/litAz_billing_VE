use sea_orm_migration::prelude::*;

/// Agrega `igtf_amount` a la tabla `invoices`.
///
/// El IGTF (Impuesto a las Grandes Transacciones Financieras) — Decreto 4.972 (julio 2024) —
/// aplica al 3% sobre pagos en divisas cuando el vendedor es Sujeto Pasivo Especial (SPE).
///
/// Se almacena en la factura para:
/// - Auditabilidad histórica (independiente del estado SPE actual de la empresa)
/// - Reportes fiscales correctos ante el SENIAT
/// - Generación exacta del PDF de la factura
///
/// Valor 0 cuando no aplica (empresa no es SPE o moneda es VES).
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240001_000020_add_igtf_to_invoices"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Invoices::Table)
                    .add_column(
                        ColumnDef::new(Invoices::IgtfAmount)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Invoices::Table)
                    .drop_column(Invoices::IgtfAmount)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Invoices {
    Table,
    IgtfAmount,
}
