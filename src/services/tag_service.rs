use crate::models::{image_tag, tag};
use crate::services::connection_db::get_connection;
use crate::services::tag_service::image_tag::Entity;
use sea_orm::{
    prelude::*, ColumnTrait, DbErr, EntityTrait, JoinType, QueryFilter, QuerySelect,
    Set,
};
use std::collections::{HashMap, HashSet};
use crate::dtos::tag_dto::TagDTO;
use crate::models::tag_color::TagColor;
use crate::models::tag::Model;

pub async fn get_tags_for_images(
    image_ids: &[i64],
    db: &DatabaseConnection,
) -> Result<HashMap<i64, HashSet<TagDTO>>, DbErr> {
    if image_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = image_tag::Entity::find()
        .join(JoinType::InnerJoin, image_tag::Relation::Tag.def())
        .filter(image_tag::Column::ImageId.is_in(image_ids.to_vec()))
        .select_only()
        .column(image_tag::Column::ImageId) // Adicione esta coluna
        .column(tag::Column::Id)
        .column(tag::Column::Name)
        .column(tag::Column::Color)
        .into_tuple::<(i64, i64, String, TagColor)>() // Agora inclui image_id
        .all(db)
        .await?;

    let mut tags_map: HashMap<i64, HashSet<TagDTO>> = HashMap::new();

    for (image_id, tag_id, name, color) in rows {
        let tag_dto = TagDTO {
            id: tag_id,
            name,
            color,
        };

        tags_map
            .entry(image_id)
            .or_insert_with(HashSet::new)
            .insert(tag_dto);
    }

    Ok(tags_map)
}

pub async fn update_tags(
    db: &DatabaseConnection,
    image_id: i64,
    tags: HashSet<TagDTO>,
) -> Result<(), DbErr> {
    use crate::models::image_tag;

    // Remove all tags for the image
    Entity::delete_many()
        .filter(image_tag::Column::ImageId.eq(image_id))
        .exec(db)
        .await?;

    // Add new tags
    for tag_dto in tags {
        if !tag_dto.name.is_empty() {
            let tag = match tag::Entity::find()
                .filter(tag::Column::Name.eq(&tag_dto.name))
                .one(db)
                .await?
            {
                Some(existing_tag) => existing_tag,
                None => {
                    // Cria uma nova tag se não existir
                    let new_tag = tag::ActiveModel {
                        name: Set(tag_dto.name.clone()),
                        color: Set(tag_dto.color.clone()),
                        ..Default::default()
                    };
                    new_tag.insert(db).await?
                }
            };

            // Add the tag to the image
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

pub async fn find_all() -> Result<Vec<TagDTO>, DbErr> {
    let db = get_connection().await?;
    let tags = tag::Entity::find()
        .all(&db)
        .await?;

    Ok(to_dto(tags))
}

pub async fn save(name: &String, color: TagColor) -> Result<(), DbErr> {
    // Convert tag name to lowercase to ensure consistency
    let name = name.to_lowercase();
    let db = get_connection().await?;
    let new_tag = tag::ActiveModel {
        name: Set(name),
        color: Set(color),
        ..Default::default()
    };
    new_tag.insert(&db).await?;
    Ok(())
}

fn to_dto(tags: Vec<Model>) -> Vec<TagDTO> {
    tags.into_iter()
        .map(|tag| TagDTO {
            id: tag.id,
            name: tag.name,
            color: tag.color,
        })
        .collect()
}