use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Adicionar a coluna com default false
        manager
            .alter_table(
                Table::alter()
                    .table(Images::Table)
                    .add_column(
                        ColumnDef::new(Images::IsPrepared)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        // Atualizar registros existentes para true
        manager
            .exec_stmt(
                Query::update()
                    .table(Images::Table)
                    .value(Images::IsPrepared, true)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Images::Table)
                    .drop_column(Images::IsPrepared)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Images {
    Table,
    IsPrepared,
}