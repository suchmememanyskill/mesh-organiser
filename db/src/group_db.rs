use std::{cmp::Reverse, u32};
use itertools::{Itertools, join};
use indexmap::IndexMap;
use sqlx::Row;
use crate::{DbError, PaginatedResponse, db_context::DbContext, model::{FileType, Model, ModelFlags, ModelGroup, ModelGroupMeta, ResourceMeta, User}, model_db::{self, ModelFilterOptions}, random_hex_32, resource_db, util::time_now};
use strum::EnumString;

#[derive(Debug, PartialEq, EnumString)]
pub enum GroupOrderBy
{
    CreatedAsc,
    CreatedDesc,
    NameAsc,
    NameDesc,
    ModifiedAsc,
    ModifiedDesc,
}

#[derive(Default)]
pub struct GroupFilterOptions
{
    pub model_ids: Option<Vec<i64>>,
    pub group_ids: Option<Vec<i64>>,
    pub label_ids: Option<Vec<i64>>,
    pub order_by: Option<GroupOrderBy>,
    pub text_search: Option<String>,
    pub file_types: Option<Vec<FileType>>,
    pub page : u32,
    pub page_size : u32,
    pub include_ungrouped_models : bool,
    pub allow_incomplete_groups : bool,
}

// TODO: This is insanely inefficient
fn convert_model_list_to_groups(models : Vec<Model>, include_ungrouped_models : bool, group_resource_map : &IndexMap<i64, ResourceMeta>) -> Vec<ModelGroup>
{
    let mut index_map: IndexMap<i64, ModelGroup> = IndexMap::new();

    for mut model in models 
    {
        let group_meta = match model.group.take()
        {
            Some(g) => g,
            None => {
                if !include_ungrouped_models {
                    continue;
                }

                ModelGroupMeta {
                    id: model.id * -1,
                    name: model.name.clone(),
                    created: model.added.clone(),
                    resource_id: None,
                    unique_global_id: String::from(""),
                    last_modified: model.last_modified.clone(),
                }
            }
        };

        let group = index_map.entry(group_meta.id).or_insert({
            let mut meta = ModelGroup::from_meta(group_meta);

            if let Some(resource_meta) = group_resource_map.get(&meta.meta.id) {
                meta.resource = Some(resource_meta.clone());
            }

            meta
        });
        // TODO: Figure out a better way to do this
        group.flags |= unsafe { ModelFlags::from_bits(model.flags.bits()).unwrap_unchecked() };

        for label in &model.labels {
            if group.labels.iter().any(|f| f.id == label.id)
            {
                continue;
            }

            group.labels.push(label.clone());
        }

        group.models.push(model);
    }

    index_map.into_values().collect()
}

// TODO: This should probably not return the entire model group, but just the meta and counts
pub async fn get_groups(db: &DbContext, user : &User, options : GroupFilterOptions) -> Result<PaginatedResponse<ModelGroup>, DbError> {
    let filtered_on_labels = options.label_ids.is_some();
    let filtered_on_text = options.text_search.is_some();
    let filtered_on_models = options.model_ids.is_some();

    let group_resource_map = resource_db::get_group_id_to_resource_map(db, user).await?;

    let models = model_db::get_models(db, user, ModelFilterOptions {
        model_ids: options.model_ids,
        group_ids: options.group_ids,
        label_ids: options.label_ids,
        text_search: options.text_search,
        page: 1,
        page_size: u32::MAX,
        file_types: options.file_types,
        ..Default::default()
    }).await?;

    let mut groups = convert_model_list_to_groups(models.items, options.include_ungrouped_models, &group_resource_map);

    // It's possible we don't have the entire group here. Re-fetching groups
    if (filtered_on_labels || filtered_on_text || filtered_on_models) && !options.allow_incomplete_groups {
        let group_ids : Vec<i64> = groups.iter().filter(|f| f.meta.id >= 0).map(|f| f.meta.id).collect();
        let fake_models : Vec<ModelGroup> = groups.into_iter().filter(|f| f.meta.id < 0).collect();

        let models = model_db::get_models(db, user, ModelFilterOptions { 
            group_ids: Some(group_ids), 
            page: 1, 
            page_size: u32::MAX,
            ..Default::default()
        }).await?;

        // TODO: Make option to split off non-complete groups into their own groups

        groups = convert_model_list_to_groups(models.items, false, &group_resource_map);
        groups.extend(fake_models);
    }

    match options.order_by.unwrap_or(GroupOrderBy::CreatedDesc) {
        GroupOrderBy::CreatedAsc => groups.sort_by_cached_key(|f| f.meta.created.clone()),
        GroupOrderBy::CreatedDesc => groups.sort_by_cached_key(|f| Reverse(f.meta.created.clone())),
        GroupOrderBy::NameAsc => groups.sort_by_cached_key(|f| f.meta.name.clone()),
        GroupOrderBy::NameDesc => groups.sort_by_cached_key(|f| Reverse(f.meta.name.clone())),
        GroupOrderBy::ModifiedAsc => groups.sort_by_cached_key(|f| f.meta.last_modified.clone()),
        GroupOrderBy::ModifiedDesc => groups.sort_by_cached_key(|f| Reverse(f.meta.last_modified.clone())),
    }

    let offset = ((options.page as u32 - 1) * options.page_size as u32) as usize;

    Ok(PaginatedResponse {
        items: groups.into_iter().skip(offset).take(options.page_size as usize).collect(),
        page: options.page,
        page_size: options.page_size
    })
}

async fn get_unique_id_from_group_id(db: &DbContext, group_id: i64) -> Result<String, DbError>
{
    let row = sqlx::query!(
        "SELECT group_unique_global_id FROM models_group WHERE group_id = ?",
        group_id
    )
    .fetch_one(db)
    .await?;

    Ok(row.group_unique_global_id)
}

async fn get_unqiue_ids_from_group_ids(db: &DbContext, group_ids: &[i64]) -> Result<IndexMap<i64, String>, DbError>
{
    let mut id_map = IndexMap::new();
    let ids = join(group_ids.iter(), ",");

    let query = format!(
        "SELECT group_id, group_unique_global_id FROM models_group WHERE group_id IN ({})",
        ids
    );

    let rows = sqlx::query(
        &query
    )
    .fetch_all(db)
    .await?;

    for row in rows {
        id_map.insert(row.get("group_id"), row.get("group_unique_global_id"));
    }

    Ok(id_map)
}

pub async fn set_group_id_on_models(
    db: &DbContext,
    user: &User,
    group_id: Option<i64>,
    model_ids: Vec<i64>,
    update_timestamp : Option<&str>
) -> Result<(), DbError> {
    // TODO: Remove clone
    let now = time_now();
    let timestamp = update_timestamp.unwrap_or(&now);
    let models = model_db::get_models_via_ids(db, user, model_ids.clone()).await?;
    let mut old_group_ids: Vec<i64> = models.iter().filter_map(|m| m.group.as_ref().map(|g| g.id)).unique().collect();
    let mut group_ids = get_unqiue_ids_from_group_ids(db, &old_group_ids).await?;
    
    if group_ids.len() != old_group_ids.len() {
        return Err(DbError::RowNotFound);
    }

    if let Some(gid) = group_id {
        let hex = get_unique_id_from_group_id(db, gid).await?;
        group_ids.insert(gid, hex);
        old_group_ids.push(gid);
    }

    let ids_placeholder = join(model_ids.iter(), ",");

    let formatted_query = format!(
        "UPDATE models
         SET model_group_id = ?
         WHERE model_id IN ({})",
        ids_placeholder
    );

    sqlx::query(&formatted_query)
        .bind(group_id)
        .execute(db)
        .await?;

    set_last_updated_on_groups(db, user, &old_group_ids, timestamp).await?;

    Ok(())
}

pub async fn add_empty_group(db: &DbContext, user : &User, group_name: &str, update_timestamp : Option<&str>) -> Result<i64, DbError> {
    let now = time_now();
    let timestamp = update_timestamp.unwrap_or(&now);
    let unique_global_id = random_hex_32();
    
    let result = sqlx::query!(
        "INSERT INTO models_group (group_name, group_created, group_user_id, group_last_modified, group_unique_global_id) VALUES (?, ?, ?, ?, ?)",
        group_name,
        now,
        user.id,
        timestamp,
        unique_global_id
    )
    .execute(db)
    .await?;

    let group_id = result.last_insert_rowid();
    Ok(group_id)
}

pub async fn edit_group(db: &DbContext, user : &User, group_id: i64, group_name: &str, update_timestamp : Option<&str>) -> Result<(), DbError> {
    let now = time_now();
    let timestamp = update_timestamp.unwrap_or(&now);

    sqlx::query!(
        "UPDATE models_group SET group_name = ?, group_last_modified = ? WHERE group_id = ? AND group_user_id = ?",
        group_name,
        timestamp,
        group_id,
        user.id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn edit_group_global_id(db: &DbContext, user : &User, group_id: i64, unique_global_id: &str) -> Result<(), DbError> {
    if unique_global_id.len() != 32 {
        return Err(DbError::InvalidArgument("Unique Global ID must be 32 characters long".to_string()));
    }

    sqlx::query!(
        "UPDATE models_group SET group_unique_global_id = ? WHERE group_id = ? AND group_user_id = ?",
        unique_global_id,
        group_id,
        user.id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn delete_group(db: &DbContext, user : &User, group_id: i64) -> Result<(), DbError> {
    sqlx::query!(
        "DELETE FROM models_group WHERE group_id = ? AND group_user_id = ?",
        group_id,
        user.id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn delete_dead_groups(db: &DbContext) -> Result<(), DbError> {
    let dead_group_ids = sqlx::query!(
        "SELECT group_id, group_user_id FROM models_group
         WHERE group_id NOT IN (SELECT DISTINCT model_group_id FROM models WHERE model_group_id IS NOT NULL)"
    )
    .fetch_all(db)
    .await?;

    for row in dead_group_ids {
        delete_group(db, &User { id: row.group_user_id.unwrap(), ..Default::default()}, row.group_id.unwrap()).await?;
    }

    Ok(())
}

pub async fn get_group_count(db: &DbContext, user : &User, include_ungrouped_models : bool) -> Result<usize, DbError> {
    let mut group_count = 0;

    let group_query = sqlx::query!(
        "SELECT COUNT(DISTINCT model_group_id) as count FROM models WHERE model_user_id = ?",
        user.id
    )
    .fetch_one(db)
    .await?;

    group_count += group_query.count as usize;

    if include_ungrouped_models {
        let ungrouped_query = sqlx::query!(
            "SELECT COUNT(*) as count FROM models WHERE model_user_id = ? AND model_group_id IS NULL",
            user.id
        )
        .fetch_one(db)
        .await?;

        group_count += ungrouped_query.count as usize;
    }

    Ok(group_count)
}

pub async fn get_group_via_id(db: &DbContext, user : &User, group_id: i64) -> Result<Option<ModelGroup>, DbError> {
    let group_resource_map = resource_db::get_group_id_to_resource_map(db, user).await?;

    let models = model_db::get_models(db, user, ModelFilterOptions {
        group_ids: Some(vec![group_id]),
        page: 1,
        page_size: u32::MAX,
        ..Default::default()
    }).await?;

    let mut groups = convert_model_list_to_groups(models.items, false, &group_resource_map);

    if groups.is_empty() {
        return Ok(None);
    }

    Ok(Some(groups.remove(0)))
}

pub async fn set_last_updated_on_group(db: &DbContext, user: &User, group_id: i64, timestamp: &str) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE models_group SET group_last_modified = ? WHERE group_id = ? AND group_user_id = ?",
        timestamp,
        group_id,
        user.id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn set_last_updated_on_groups(db: &DbContext, user: &User, group_ids: &[i64], timestamp: &str) -> Result<(), DbError> {
    let ids_placeholder = join(group_ids.iter(), ",");

    let formatted_query = format!(
        "UPDATE models_group
         SET group_last_modified = ?
         WHERE group_id IN ({}) AND group_user_id = ?",
        ids_placeholder
    );

    sqlx::query(&formatted_query)
        .bind(timestamp)
        .bind(user.id)
        .execute(db)
        .await?;

    Ok(())
}