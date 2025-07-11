use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::image_tag::Entity")]
    ImageTag,
}

impl Related<super::image::Entity> for Entity {
    fn to() -> RelationDef {
        super::image_tag::Relation::Image.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::image_tag::Relation::Tag.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}