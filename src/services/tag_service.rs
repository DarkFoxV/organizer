use crate::models::{image_tag, tag};
use crate::services::connection_db::get_connection;
use crate::services::tag_service::image_tag::Entity;
use sea_orm::{
    prelude::*, ColumnTrait, DbErr, EntityTrait, JoinType, QueryFilter, QuerySelect,
    Set,
};
use std::collections::HashSet;

pub async fn get_tags_for_images(
    image_ids: &[i64],
    db: &DatabaseConnection,
) -> Result<Vec<(i64, String)>, DbErr> {
    if image_ids.is_empty() {
        return Ok(Vec::new());
    }

    image_tag::Entity::find()
        .select_only()
        .column(image_tag::Column::ImageId)
        .column(tag::Column::Name)
        .join(JoinType::InnerJoin, image_tag::Relation::Tag.def())
        .filter(image_tag::Column::ImageId.is_in(image_ids.to_vec()))
        .into_tuple::<(i64, String)>()
        .all(db)
        .await
}


pub async fn update_tags(
    db: &DatabaseConnection,
    image_id: i64,
    tags: HashSet<String>,
) -> Result<(), DbErr> {

    use crate::models::image_tag;

    Entity::delete_many()
        .filter(image_tag::Column::ImageId.eq(image_id))
        .exec(db)
        .await?;

    for tag_name in tags {
        if !tag_name.is_empty() {

            let tag = match tag::Entity::find()
                .filter(tag::Column::Name.eq(&tag_name))
                .one(db)
                .await?
            {
                Some(existing_tag) => existing_tag,
                None => {
                    // Cria uma nova tag se não existir
                    let new_tag = tag::ActiveModel {
                        name: Set(tag_name.clone()),
                        ..Default::default()
                    };
                    new_tag.insert(db).await?
                }
            };

            // Cria a relação image_tag
            let image_tag_model = image_tag::ActiveModel {
                image_id: Set(image_id),
                tag_id: Set(tag.id),
                ..Default::default()
            };
            image_tag_model.insert(db).await?;
        }
    }

    Ok(())
}

pub async fn find_all() -> Result<Vec<String>, DbErr> {
    let db = get_connection().await?;
    tag::Entity::find()
        .select_only()
        .column(tag::Column::Name)
        .into_tuple::<String>()
        .all(&db)
        .await
}

pub async fn save(p0: &String) -> Result<(), DbErr> {
    // Convert tag name to lowercase to ensure consistency
    let name = p0.to_lowercase();
    let db = get_connection().await?;
    let new_tag = tag::ActiveModel {
        name: Set(name),
        ..Default::default()
    };
    new_tag.insert(&db).await?;
    Ok(())
}
