use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Images::Table)
                    .add_column(
                        ColumnDef::new(Images::IsFolder)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Images::Table)
                    .drop_column(Images::IsFolder)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Images {
    Table,
    IsFolder,
}
