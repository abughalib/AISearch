use crate::constants::*;
use crate::embedding::get_embeddings;
use crate::vars::get_pgurl;
use anyhow::{Error, Result};
use async_once::AsyncOnce;
use chrono::NaiveDateTime;
use futures::TryStreamExt;
use lazy_static::lazy_static;
use pgvector::Vector;
use serde_json::Value;
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

lazy_static! {
    static ref POOL: AsyncOnce<Result<PgPool>> =
        AsyncOnce::new(async { create_connection().await });
}

async fn create_connection() -> Result<PgPool> {
    match PgPoolOptions::new()
        .max_connections(MAX_POOL_CONNECTION)
        .connect(&get_pgurl())
        .await
    {
        Ok(pool) => {
            sqlx::query(CREATE_VECTOR_SQL).execute(&pool).await?;

            sqlx::query(&create_search_base_sql())
                .execute(&pool)
                .await?;

            return Ok(pool);
        }
        Err(e) => {
            println!("{:?}", e);

            return Err(e.into());
        }
    };
}

#[derive(sqlx::FromRow, Clone)]
pub struct EmbeddingVectorValue {
    pub id: i64,
    pub content_id: String,
    pub content_chunk: String,
    pub chunk_number: i32,
    pub embedding: Vector,
    pub metadata: Value,
    pub create_at: NaiveDateTime,
}

impl EmbeddingVectorValue {
    pub async fn get_adjacent_chunks(
        &self,
        table_name: &str,
        mut upper: i32,
        lower: i32,
    ) -> Result<Vec<EmbeddingVectorValue>> {
        if let Ok(pool) = POOL.get().await {
            if upper > self.chunk_number {
                upper = self.chunk_number;
            }

            let result = sqlx::query_as::<_, EmbeddingVectorValue>(&get_adj_chunk_sql(table_name))
                .bind(&self.content_id)
                .bind(self.chunk_number - upper)
                .bind(self.chunk_number + lower)
                .fetch_all(pool)
                .await?;

            return Ok(result);
        }
        return Err(Error::msg("DB Connection Initialization Failed."));
    }
}

pub struct RowContent {
    pub id: i32,
    pub title: String,
    pub text: String,
    pub created_at: NaiveDateTime,
}

pub async fn create_table(table_name: &str) -> Result<()> {
    if let Ok(pool) = POOL.get().await {
        sqlx::query(&create_vector_table_sql(table_name))
            .execute(pool)
            .await?;

        sqlx::query(&create_raw_content_table_sql(table_name))
            .execute(pool)
            .await?;

        let _ = sqlx::query(&insert_into_search_table_sql())
            .bind(table_name)
            .execute(pool)
            .await;

        return Ok(());
    }

    Err(Error::msg("DB Connection Initialization Failed."))
}

async fn insert_into(table_name: &str, values: EmbeddingVectorValue) -> Result<()> {
    if let Ok(pool) = POOL.get().await {
        sqlx::query(&insert_into_vector_table_sql(table_name))
            .bind(values.content_id)
            .bind(values.content_chunk)
            .bind(values.chunk_number)
            .bind(values.embedding)
            .bind(values.metadata)
            .bind(values.create_at)
            .execute(pool)
            .await?;

        return Ok(());
    }

    return Err(Error::msg("DB Connection Intialization Failed."));
}

async fn builk_insert_into(
    table_name: &str,
    content_ids: Vec<String>,
    content_chunks: Vec<String>,
    chunk_numbers: Vec<i32>,
    embeddings: Vec<Vector>,
    metadatas: Vec<Value>,
    created_ats: Vec<NaiveDateTime>,
) -> Result<()> {
    if let Ok(pool) = POOL.get().await {
        sqlx::query(&bulk_insert_into_vector_table_sql(table_name))
            .bind(content_ids)
            .bind(content_chunks)
            .bind(chunk_numbers)
            .bind(embeddings)
            .bind(metadatas)
            .bind(created_ats)
            .execute(pool)
            .await
            .map_err(Error::msg)?;
    }

    Ok(())
}

async fn insert_content_into(
    table_name: &str,
    content_id: &str,
    title: &str,
    text: &str,
    metadata: Value,
) -> Result<()> {
    if let Ok(pool) = POOL.get().await {
        sqlx::query(&insert_raw_content_sql(table_name))
            .bind(content_id)
            .bind(title)
            .bind(text)
            .bind(metadata)
            .execute(pool)
            .await?;

        return Ok(());
    }

    return Err(Error::msg("DB Connection Intialization Failed."));
}

pub async fn insert_vector_index_pg(
    table_name: &str,
    content_id: &str,
    chunk_number: i32,
    content_chunk: &str,
    metadata: Value,
) -> Result<()> {
    let content_chunk = content_chunk
        .chars()
        .filter(|c| c.is_ascii())
        .collect::<String>();

    if content_chunk.is_empty() {
        return Err(anyhow::anyhow!("Content Chunk is empty"));
    }

    let vector: Vec<f32> = get_embeddings(&content_chunk)?
        .reshape((EMBEDDING_DIMENSION,))?
        .to_vec1()?;

    let values = EmbeddingVectorValue {
        id: 0,
        content_id: content_id.to_string(),
        content_chunk: content_chunk.to_string(),
        chunk_number,
        metadata,
        embedding: vector.into(),
        create_at: NaiveDateTime::default(),
    };

    insert_into(table_name, values).await?;

    Ok(())
}

pub async fn insert_split_chunks(
    table_name: &str,
    title: &str,
    text: &str,
    metadata: Value,
) -> Result<()> {
    let content_id = Uuid::new_v4().to_string().replace("-", "");

    insert_content_into(table_name, &content_id, title, text, metadata.clone()).await?;

    let mut chunks = text.split("\n").collect::<Vec<&str>>();
    chunks.retain(|c| !c.is_empty());

    let overlap_size: usize = 150;

    let mut position = 0;
    let mut chunks_with_overlap = Vec::new();

    while position < text.len() {
        let end = std::cmp::min(position + 1000, text.len());
        let mut chunk = &text[position..end];

        if end != text.len() {
            let overlap_end = std::cmp::min(end + overlap_size, text.len());
            chunk = &text[position..overlap_end];

            position = overlap_end - overlap_size;
        } else {
            position = end;
        }

        chunks_with_overlap.push(chunk);
    }

    for (i, chunk) in chunks_with_overlap.clone().into_iter().enumerate() {
        insert_vector_index_pg(table_name, &content_id, i as i32, chunk, metadata.clone()).await?;
    }

    Ok(())
}

pub async fn builk_insert_split_chunks(
    table_name: &str,
    title: &str,
    text: &str,
    metadata: Value,
) -> Result<()> {
    let content_id = Uuid::new_v4().to_string().replace("-", "");

    insert_content_into(table_name, &content_id, title, text, metadata.clone()).await?;

    let mut chunks = text.split("\n").collect::<Vec<&str>>();
    chunks.retain(|c| !c.is_empty());

    let overlap_size: usize = 150;

    let mut position = 0;
    let mut chunks_with_overlap = Vec::new();

    while position < text.len() {
        let end = std::cmp::min(position + 1000, text.len());
        let mut chunk = &text[position..end];

        if end != text.len() {
            let overlap_end = std::cmp::min(end + overlap_size, text.len());
            chunk = &text[position..overlap_end];

            position = overlap_end - overlap_size;
        } else {
            position = end;
        }

        chunks_with_overlap.push(chunk);
    }

    let mut content_ids: Vec<String> = Vec::new();
    let mut content_chunks: Vec<String> = Vec::new();
    let mut chunk_numbers: Vec<i32> = Vec::new();
    let mut metadatas: Vec<Value> = Vec::new();
    let mut embeddings: Vec<Vector> = Vec::new();
    let mut created_ats: Vec<NaiveDateTime> = Vec::new();

    for (i, chunk) in chunks_with_overlap.clone().into_iter().enumerate() {
        let content_chunk: String = chunk.chars().filter(|c| c.is_ascii()).collect::<String>();
        let content_chunk = content_chunk.trim();

        if content_chunk.is_empty() {
            continue;
        }

        let vector: Vec<f32> = get_embeddings(&content_chunk)?
            .reshape((EMBEDDING_DIMENSION,))?
            .to_vec1()?;

        content_ids.push(content_id.to_owned());
        content_chunks.push(content_chunk.to_owned());
        chunk_numbers.push(i as i32);
        metadatas.push(metadata.clone());
        embeddings.push(vector.into());
        created_ats.push(NaiveDateTime::default());
    }

    builk_insert_into(
        table_name,
        content_ids,
        content_chunks,
        chunk_numbers,
        embeddings,
        metadatas,
        created_ats,
    )
    .await
    .map_err(Error::msg)?;

    Ok(())
}

pub async fn get_similar_results(
    table_name: &str,
    query: Vector,
    max_similar_res: usize,
) -> Result<Vec<EmbeddingVectorValue>> {
    if let Ok(pool) = POOL.get().await {
        let res = sqlx::query_as::<_, EmbeddingVectorValue>(&get_similar_result_query(
            table_name,
            max_similar_res,
        ))
        .bind(query)
        .fetch_all(pool)
        .await?;

        return Ok(res);
    }

    return Err(Error::msg("DB Connection Intialization Failed."));
}

pub async fn list_search_tables() -> Result<Vec<String>> {
    if let Ok(pool) = POOL.get().await {
        let query: String = get_search_tables_sql();

        let mut rows = sqlx::query(&query).fetch(pool);

        let mut table_names: Vec<String> = Vec::new();

        while let Some(row) = rows.try_next().await? {
            let table_name: &str = row.try_get("table_name")?;
            table_names.push(table_name.to_string())
        }

        return Ok(table_names);
    }

    return Err(Error::msg("DB Connection Initialization Failed."));
}

pub async fn delete_table(table_name: &str) -> Result<()> {
    if let Ok(pool) = POOL.get().await {
        sqlx::query(&get_drop_table_sql(table_name.to_owned()))
            .execute(pool)
            .await?;

        sqlx::query(&get_drop_table_sql(table_name.to_owned() + &"_content"))
            .execute(pool)
            .await?;

        sqlx::query(&get_delete_from_search_table_sql(table_name))
            .execute(pool)
            .await?;

        return Ok(());
    }

    return Err(Error::msg("DB Connection Initialization Failed."));
}
