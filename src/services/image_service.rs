use crate::dtos::image_dto::{ImageDTO, ImageUpdateDTO};
use crate::dtos::tag_dto::TagDTO;
use crate::models::filter::{Filter, SortOrder};
use crate::models::image::{ActiveModel, Entity, Model};
use crate::models::page::Page;
use crate::models::{image, image_tag, tag};
use crate::services::connection_db::db_ref;
use crate::services::tag_service::{get_tags_for_images, update_tags_for_image};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, InsertResult, JoinType, Order,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait, prelude::*,
};
use std::collections::{HashMap, HashSet};

pub async fn insert_image(desc: &str) -> Result<i64, DbErr> {
    let db = db_ref();
    let new_image = ActiveModel {
        description: Set(desc.to_string()),
        path: Set(String::new()),
        thumbnail_path: Set(String::new()),
        is_prepared: Set(false),
        ..Default::default()
    };

    let result: InsertResult<ActiveModel> = Entity::insert(new_image).exec(db).await?;
    Ok(result.last_insert_id)
}

pub async fn find_all(filter: Filter, page: u64, size: u64) -> Result<Page<ImageDTO>, DbErr> {
    let db = db_ref();
    // Verify if we have a query
    let has_query = !filter.query.trim().is_empty();
    let has_tags = !filter.tags.is_empty();

    // If we don't have a query or tags, just return all
    if !has_query && !has_tags {
        return find_all_images_without_filter(page, size, filter, db).await;
    }

    // Base query for images
    let mut query = image::Entity::find();

    // If we have a query, apply it
    if has_tags {
        let tag_count = filter.tags.len() as i64;

        query = query
            .join(JoinType::InnerJoin, image::Relation::ImageTag.def())
            .join(JoinType::InnerJoin, image_tag::Relation::Tag.def())
            .filter(tag::Column::Name.is_in(filter.tags.iter().cloned().collect::<Vec<_>>()))
            .group_by(image::Column::Id)
            .having(Expr::col(tag::Column::Name).count().eq(tag_count));
    }

    // Apply conditions to query
    if let Some(desc_cond) = build_desc_condition(&filter.query) {
        query = query.filter(desc_cond);
    }

    // Count total
    let total_count = query
        .clone()
        .select_only()
        .column(image::Column::Id)
        .distinct()
        .count(db)
        .await?;

    let total_pages = if total_count == 0 {
        0
    } else {
        (total_count + size - 1) / size
    };

    if filter.sort_order == SortOrder::CreatedDesc {
        query = query.order_by(image::Column::CreatedAt, Order::Desc);
    } else {
        query = query.order_by(image::Column::CreatedAt, Order::Asc);
    }

    // Search for images
    let images = query
        .distinct()
        .limit(size)
        .offset(page * size)
        .into_model::<Model>()
        .all(db)
        .await?;

    // Search for tags for each image
    let image_ids: Vec<i64> = images.iter().map(|img| img.id).collect();

    let tags_map = get_tags_for_images(&image_ids, db).await?;

    let dtos = to_dto(images, tags_map);

    Ok(Page {
        content: dtos,
        total_pages,
        page_number: page,
    })
}

async fn find_all_images_without_filter(
    page: u64,
    size: u64,
    filter: Filter,
    db: &DatabaseConnection,
) -> Result<Page<ImageDTO>, DbErr> {
    // Count total
    let total_count = image::Entity::find().count(db).await?;
    let total_pages = if total_count == 0 {
        0
    } else {
        (total_count + size - 1) / size
    };

    let mut query = image::Entity::find().limit(size).offset(page * size);

    query = if filter.sort_order == SortOrder::CreatedDesc {
        query.order_by(image::Column::CreatedAt, Order::Desc)
    } else {
        query.order_by(image::Column::CreatedAt, Order::Asc)
    };

    let images: Vec<Model> = query.all(db).await?;

    // Search for tags for each image
    let image_ids: Vec<i64> = images.iter().map(|img| img.id).collect();

    let tags_map = get_tags_for_images(&image_ids, db).await?;

    let dtos = to_dto(images, tags_map);

    Ok(Page {
        content: dtos,
        total_pages,
        page_number: page,
    })
}

pub async fn delete_image(id_val: i64) -> Result<(), DbErr> {
    let db = db_ref();
    let txn = db.begin().await?;

    Entity::delete_by_id(id_val).exec(&txn).await?;

    txn.commit().await?;

    // Return Ok regardless if deletion happened or not
    Ok(())
}

pub async fn update_from_dto(id: i64, dto: ImageUpdateDTO) -> Result<Model, DbErr> {
    let db = db_ref();
    let existing_model = Entity::find_by_id(id)
        .one(&*db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("Image not found".to_string()))?;

    let mut active_model: ActiveModel = existing_model.into();

    if let Some(path) = dto.path {
        if !path.is_empty() {
            active_model.path = Set(path);
        }
    }

    if let Some(thumbnail_path) = dto.thumbnail_path {
        if !thumbnail_path.is_empty() {
            active_model.thumbnail_path = Set(thumbnail_path);
        }
    }

    if let Some(description) = dto.description {
        if !description.is_empty() {
            active_model.description = Set(description);
        }
    }

    active_model.is_prepared = Set(dto.is_prepared);

    active_model.is_folder = Set(dto.is_folder);

    let updated_model = active_model.update(db).await?;

    if let Some(tags) = dto.tags {
        if !tags.is_empty() {
            update_tags_for_image(db, id, tags).await?;
        }
    }

    Ok(updated_model)
}

#[allow(dead_code)]
pub async fn find_by_id(id_val: i64) -> Result<Option<ImageDTO>, DbErr> {
    let db = db_ref();
    // Consulta o Model da imagem diretamente, sem recursão
    if let Some(model) = Entity::find_by_id(id_val).one(db).await? {
        // Busca as tags dessa imagem
        let tags_map: HashMap<i64, HashSet<TagDTO>> = get_tags_for_images(&[id_val], db).await?;

        // Constrói o DTO diretamente aqui
        let dto = ImageDTO {
            id: model.id,
            path: model.path,
            thumbnail_path: model.thumbnail_path,
            description: model.description,
            tags: tags_map.get(&id_val).cloned().unwrap_or_default(),
            created_at: model.created_at.format("%Y-%m-%d").to_string(),
            is_folder: model.is_folder,
            is_prepared: model.is_prepared,
        };

        Ok(Some(dto))
    } else {
        Ok(None)
    }
}

fn build_desc_condition(query: &str) -> Option<Condition> {
    let q = query.trim();
    if q.is_empty() {
        return None;
    }

    if q.contains('+') {
        let mut cond = Condition::any();
        for term in q.split('+').map(str::trim).filter(|t| !t.is_empty()) {
            cond = cond.add(image::Column::Description.contains(term));
        }
        Some(cond)
    } else {
        Some(Condition::all().add(image::Column::Description.contains(q)))
    }
}

pub fn to_dto(images: Vec<Model>, tags_map: HashMap<i64, HashSet<TagDTO>>) -> Vec<ImageDTO> {
    images
        .iter()
        .map(|img| to_image_dto(img, &tags_map))
        .collect()
}

pub fn to_image_dto(model: &Model, tags_map: &HashMap<i64, HashSet<TagDTO>>) -> ImageDTO {
    ImageDTO {
        id: model.id,
        path: model.path.clone(),
        thumbnail_path: model.thumbnail_path.clone(),
        description: model.description.clone(),
        tags: tags_map.get(&model.id).cloned().unwrap_or_default(),
        created_at: model.created_at.format("%Y-%m-%d").to_string(),
        is_folder: model.is_folder,
        is_prepared: model.is_prepared,
    }
}
