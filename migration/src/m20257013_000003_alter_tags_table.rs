use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Tags::Table)
                    .add_column(
                        ColumnDef::new(Tags::Color)
                            .string_len(10)
                            .not_null()
                            .default("blue"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Tags::Table)
                    .drop_column(Tags::Color)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Tags {
    Table,
    Color,
}