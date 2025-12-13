use indexmap::IndexMap;
use itertools::join;
use serde::de;
use sqlx::{Execute, QueryBuilder, query};
use sqlx::Row;
use strum::EnumString;
use crate::model::Blob;
use crate::util::{random_hex_32, time_now};
use crate::{DbError, PaginatedResponse, db_context::DbContext, label_db, model::{Label, LabelMeta, Model, ModelFlags, ModelGroup, ModelGroupMeta, User, convert_label_meta_list_to_map}};

#[derive(Debug, PartialEq, EnumString)]
pub enum ModelOrderBy {
    AddedAsc,
    AddedDesc,
    NameAsc,
    NameDesc,
    SizeAsc,
    SizeDesc,
    ModifiedAsc,
    ModifiedDesc,
}

impl ModelOrderBy {
    pub fn to_sql(&self) -> &'static str {
        match self {
            ModelOrderBy::AddedAsc => "model_added ASC",
            ModelOrderBy::AddedDesc => "model_added DESC",
            ModelOrderBy::NameAsc => "model_name ASC",
            ModelOrderBy::NameDesc => "model_name DESC",
            ModelOrderBy::SizeAsc => "blob_size ASC",
            ModelOrderBy::SizeDesc => "blob_size DESC",
            ModelOrderBy::ModifiedAsc => "model_last_modified ASC",
            ModelOrderBy::ModifiedDesc => "model_last_modified DESC",
        }
    }
}

#[derive(Default)]
pub struct ModelFilterOptions {
    pub model_ids: Option<Vec<i64>>,
    pub group_ids: Option<Vec<i64>>,
    pub label_ids: Option<Vec<i64>>,
    pub order_by: Option<ModelOrderBy>, 
    pub text_search: Option<String>,
    pub model_flags: Option<ModelFlags>,
    pub page : u32,
    pub page_size : u32,
}

pub async fn get_models(db: &DbContext, user : &User, options : ModelFilterOptions) -> Result<PaginatedResponse<Model>, DbError> {
    let offset = (options.page as i64 - 1) * options.page_size as i64;

    let mut query_builder = QueryBuilder::new(
        format!("SELECT models.model_id, model_name, model_url, model_desc, model_added, model_flags, model_unique_global_id, model_last_modified,
				blob_id, blob_sha256, blob_filetype, blob_size, blob_path,
                GROUP_CONCAT(labels.label_id) AS label_ids,
                models_group.group_id, group_name, group_created, group_resource_id, group_unique_global_id, group_last_modified
         FROM models 
         LEFT JOIN models_labels ON models.model_id = models_labels.model_id 
         LEFT JOIN labels ON models_labels.label_id = labels.label_id
         LEFT JOIN models_group ON models.model_group_id = models_group.group_id
		 INNER JOIN blobs ON models.model_blob_id = blobs.blob_id
         WHERE models.model_user_id = {} ", user.id)
    );

    let mut seperated = query_builder.separated(" AND ");
    seperated.push("");
    
    if let Some(model_ids) = options.model_ids
    {
        seperated.push(format!("models.model_id IN ({})", join(model_ids, ",")));
    }

    if let Some(group_ids) = options.group_ids
    {
        seperated.push(format!("group_id IN ({})", join(group_ids, ",")));
    }

    if let Some(label_ids) = options.label_ids
    {
        seperated.push(format!("labels.label_id IN ({})", join(label_ids, ",")));
    }

    if let Some(model_flags) = options.model_flags
    {
        seperated.push(format!("(models.model_flags & {}) = {}", model_flags.bits(), model_flags.bits()));
    }

    if let Some(text_search) = options.text_search
    {
        let str = format!("%{}%", text_search);
        seperated.push("(model_name LIKE ");
        seperated.push_bind_unseparated(str.clone());
        seperated.push_unseparated(" OR model_desc LIKE ");
        seperated.push_bind_unseparated(str.clone());
        seperated.push_unseparated(" OR group_name LIKE ");
        seperated.push_bind_unseparated(str);
        seperated.push_unseparated(")");
    }

    query_builder.push(" GROUP BY models.model_id ");

    if let Some(order_by) = options.order_by {
        query_builder.push(format!("ORDER BY {} ", order_by.to_sql()));
    }

    query_builder.push(format!("LIMIT {} OFFSET {}", options.page_size, offset));

    let query = query_builder.build();

    #[cfg(debug_assertions)]
    println!("Generated SQL Query: {}", query.sql());

    let rows = query.fetch_all(db).await?;
    let mut models = Vec::with_capacity(rows.len());

    let min_labels = label_db::get_labels_min(db).await?;
    let min_labels_map = convert_label_meta_list_to_map(min_labels);

    for row in rows {
        models.push(Model {
            id: row.get("model_id"),
            name: row.get("model_name"),
            last_modified: row.get("model_last_modified"),
            blob: Blob {
                id: row.get("blob_id"),
                sha256: row.get("blob_sha256"),
                filetype: row.get("blob_filetype"),
                size: row.get("blob_size"),
                added: row.get("model_added"),
                disk_path: row.get("blob_path"),
            },
            link: row.get("model_url"),
            description: row.get("model_desc"),
            added: row.get("model_added"),
            group: match row.get::<Option<i64>, _>("group_id") {
                Some(id) => Some(ModelGroupMeta {
                    id: id,
                    name: row.get("group_name"),
                    created: row.get("group_created"),
                    resource_id: row.get("group_resource_id"),
                    unique_global_id: row.get("group_unique_global_id"),
                    last_modified: row.get("group_last_modified"),
                }),
                None => None,
            },
            labels: match row.get::<Option<String>, _>("label_ids") {
                Some(label_ids) => {
                    let label_ids = label_ids.split(',').filter_map(|s| s.parse::<i64>().ok()).collect::<Vec<i64>>();
                    label_ids.iter().filter_map(|id| min_labels_map.get(id).cloned()).collect()
                },
                None => Vec::new(),
            },
            flags: ModelFlags::from_bits(row.get::<i64, _>("model_flags") as u32).unwrap_or(ModelFlags::empty()),
            unique_global_id: row.get("model_unique_global_id"),
        })
    }

    return Ok(PaginatedResponse {
        page: options.page,
        page_size: options.page_size,
        items: models,
    });
}

pub async fn get_models_via_ids(db: &DbContext, user: &User, ids: Vec<i64>) -> Result<Vec<Model>, DbError> {
    let options = ModelFilterOptions {
        model_ids: Some(ids),
        page: 1,
        page_size: u32::MAX,
        ..Default::default()
    };

    let paginated_response = get_models(db, user, options).await?;
    Ok(paginated_response.items)
}

pub async fn add_model(db: &DbContext, user: &User, name: &str, blob_id: i64, link: Option<&str>, update_timestamp : Option<&str>) -> Result<i64, DbError>
{
    let now = time_now();
    let timestamp = update_timestamp.unwrap_or(&now);
    let hex = random_hex_32();
    
    let result = sqlx::query!(
        "INSERT INTO models (model_name, model_blob_id, model_added, model_url, model_user_id, model_unique_global_id, model_last_modified)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        name,
        blob_id,
        now,
        link,
        user.id,
        hex,
        timestamp,
    )
    .execute(db)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn edit_model(db: &DbContext, user: &User, id: i64, name: &str, link: Option<&str>, description: Option<&str>, flags: ModelFlags, update_timestamp : Option<&str>) -> Result<(), DbError>
{
    let now = time_now();
    let timestamp = update_timestamp.unwrap_or(&now);
    let flags = flags.bits() as i64;
    sqlx::query!(
        "UPDATE models SET model_name = ?, model_url = ?, model_desc = ?, model_flags = ?, model_last_modified = ? WHERE model_id = ? AND model_user_id = ?",
        name,
        link,
        description,
        flags,
        timestamp,
        id,
        user.id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn edit_model_global_id(db: &DbContext, user: &User, id: i64, unique_global_id: &str) -> Result<(), DbError>
{
    if unique_global_id.len() != 32 {
        return Err(DbError::InvalidArgument("Unique Global ID must be 32 characters long".to_string()));
    }

    sqlx::query!(
        "UPDATE models SET model_unique_global_id = ? WHERE model_id = ? AND model_user_id = ?",
        unique_global_id,
        id,
        user.id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn delete_model(db: &DbContext, user: &User, id: i64) -> Result<(), DbError>
{
    sqlx::query!(
        "DELETE FROM models WHERE model_id = ? AND model_user_id = ?",
        id,
        user.id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn delete_models(db: &DbContext, user: &User, ids: &[i64]) -> Result<(), DbError>
{
    if ids.is_empty() {
        return Ok(());
    }

    let ids_placeholder = join(ids.iter(), ",");

    let query = format!(
        "DELETE FROM models WHERE model_user_id = ? AND model_id IN ({})",
        ids_placeholder
    );

    sqlx::query(&query)
        .bind(user.id)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn get_unique_id_from_model_id(db: &DbContext, model_id: i64) -> Result<String, DbError>
{
    let row = sqlx::query!(
        "SELECT model_unique_global_id FROM models WHERE model_id = ?",
        model_id
    )
    .fetch_one(db)
    .await?;

    Ok(row.model_unique_global_id)
}

pub async fn get_unique_ids_from_model_ids(db: &DbContext, model_ids: Vec<i64>) -> Result<IndexMap<i64, String>, DbError>
{
    let ids_placeholder = join(model_ids.iter(), ",");

    let query = format!(
        "SELECT model_id, model_unique_global_id FROM models WHERE model_id IN ({})",
        ids_placeholder
    );

    let rows = sqlx::query(&query)
        .fetch_all(db)
        .await?;

    let mut id_map = IndexMap::new();

    for row in rows {
        let id: i64 = row.get("model_id");
        let unique_id: String = row.get("model_unique_global_id");
        id_map.insert(id, unique_id);
    }

    Ok(id_map)
}

// TODO: Can we make a get model via sha256?
pub async fn get_model_id_via_sha256(db: &DbContext, user : &User, sha256: &str) -> Result<Option<i64>, DbError> {
    let row = sqlx::query!(
        "SELECT model_id FROM models INNER JOIN blobs ON models.model_blob_id = blobs.blob_id WHERE blob_sha256 = ? AND model_user_id = ?",
        sha256,
        user.id
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(r) => Ok(Some(r.model_id.unwrap())),
        None => Ok(None),
    }
}

pub async fn get_model_count(db: &DbContext, user : &User, flags : Option<ModelFlags>) -> Result<usize, DbError> {
    let count = match flags {
        Some(f) => {
            let bits = f.bits() as i64;
            sqlx::query!(
                "SELECT COUNT(*) as count FROM models WHERE model_user_id = ? AND (models.model_flags & ?) = ?",
                user.id,
                bits,
                bits
            )
            .fetch_one(db)
            .await?.count
        },
        None => sqlx::query!(
            "SELECT COUNT(*) as count FROM models WHERE model_user_id = ?",
            user.id
        )
        .fetch_one(db)
        .await?.count
    };

    Ok(count as usize)
}

pub struct ModelSizeResult {
    pub total_size: i64,
    pub blob_sha256: Vec<String>,
}

pub async fn get_size_of_models(db: &DbContext, user : &User) -> Result<ModelSizeResult, DbError> {
    let row = sqlx::query!(
        "SELECT SUM(blob_size) as \"total_size: i64\", GROUP_CONCAT(blob_sha256) as \"blob_sha256: String\" FROM models JOIN blobs ON models.model_blob_id = blobs.blob_id WHERE model_user_id = ?",
        user.id
    )
    .fetch_one(db)
    .await?;

    Ok(ModelSizeResult { total_size: row.total_size.unwrap_or(0), blob_sha256: row.blob_sha256.map(|f| f.split(",").map(|f| f.to_string()).collect()).unwrap_or(Vec::new()) })
}