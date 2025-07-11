use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ImageTags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ImageTags::ImageId).integer().not_null())
                    .col(ColumnDef::new(ImageTags::TagId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(ImageTags::ImageId)
                            .col(ImageTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_image_tags_image_id")
                            .from(ImageTags::Table, ImageTags::ImageId)
                            .to(Images::Table, Images::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_image_tags_tag_id")
                            .from(ImageTags::Table, ImageTags::TagId)
                            .to(Tags::Table, Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ImageTags::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ImageTags {
    Table,
    ImageId,
    TagId,
}

// Referências para foreign keys
#[derive(DeriveIden)]
enum Images {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Tags {
    Table,
    Id,
}