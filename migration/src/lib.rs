mod m2025701_000001_create_images_table;
mod m2025701_000002_create_tags_table;
mod m2025701_000003_create_image_tags_table;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m2025701_000001_create_images_table::Migration),
            Box::new(m2025701_000002_create_tags_table::Migration),
            Box::new(m2025701_000003_create_image_tags_table::Migration),
        ]
    }
}
