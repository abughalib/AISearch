pub const SEARCH_TABLES: &'static str = "search_tables";
pub const EMBEDDING_DIMENSION: usize = 1024;
pub const MAX_POOL_CONNECTION: u32 = 20;
pub const CREATE_VECTOR_SQL: &'static str = "CREATE EXTENSION IF NOT EXISTS vector";

pub fn create_search_base_sql() -> String {
    let query = format!(
        "CREATE TABLE IF NOT EXISTS {SEARCH_TABLES} (id bigserial PRIMARY KEY, table_name TEXT UNIQUE)"
    );
    return query;
}

pub fn get_adj_chunk_sql(table_name: &str) -> String {
    format!(
        "SELECT * FROM {table_name} WHERE content_id = $1 AND chunk_number >= $2 AND chunk_number <= $3 ORDER BY chunk_number ASC"
    )
}

pub fn create_vector_table_sql(table_name: &str) -> String {
    format!("CREATE TABLE IF NOT EXISTS {table_name} (id bigserial PRIMARY KEY, content_id TEXT, content_chunk TEXT, chunk_number int, embedding vector({EMBEDDING_DIMENSION}), metadata JSON, created_at timestamp)")
}

pub fn create_raw_content_table_sql(table_name: &str) -> String {
    format!("CREATE TABLE IF NOT EXISTS {table_name}_content (id bigserial PRIMARY KEY, content_id TEXT, title TEXT, text TEXT, metadata JSON)")
}

pub fn insert_into_search_table_sql() -> String {
    format!("INSERT INTO {SEARCH_TABLES} (table_name) VALUES ($1)")
}

pub fn insert_into_vector_table_sql(table_name: &str) -> String {
    format!("INSERT INTO {table_name} (content_id, content_chunk, chunk_number, embedding, metadata, created_at) VALUES ($1, $2, $3, $4, $5::jsonb, $6)")
}

pub fn bulk_insert_into_vector_table_sql(table_name: &str) -> String {
    format!("
        INSERT INTO {table_name}(content_id, content_chunk, chunk_number, embedding, metadata, created_at)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::int4[], $4::vector[], $5::jsonb[], $6::timestamp[])
    ")
}

pub fn insert_raw_content_sql(table_name: &str) -> String {
    format!("INSERT INTO {table_name}_content (content_id, title, text, metadata) VALUES ($1, $2, $3, $4::jsonb)")
}

pub fn get_similar_result_query(table_name: &str, limit: usize, minimum_score: f32) -> String {
    format!(
        "SELECT *, (1.0-(embedding <=> $1::vector)) as score FROM {table_name} WHERE (1.0-(embedding <=> $1::vector)) >={minimum_score} ORDER BY score DESC LIMIT {}",
        limit
    )
}

pub fn get_search_tables_sql() -> String {
    format!("SELECT table_name FROM {SEARCH_TABLES}")
}

pub fn get_drop_table_sql(table_name: String) -> String {
    format!("DROP TABLE IF EXISTS {table_name}")
}

pub fn get_delete_from_search_table_sql(table_name: &str) -> String {
    format!("DELETE FROM {SEARCH_TABLES} WHERE table_name = '{table_name}'")
}
