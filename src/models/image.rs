use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub path: String,
    pub thumbnail_path: String,
    pub description: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::image_tag::Entity")]
    ImageTag,
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        super::image_tag::Relation::Tag.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::image_tag::Relation::Image.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}