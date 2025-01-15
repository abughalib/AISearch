pub const SEARCH_TABLES: &'static str = "search_tables";
pub const EMBEDDING_DIMENSION: usize = 1024;
pub const MAX_POOL_CONNECTION: u32 = 20;
pub const CREATE_VECTOR_SQL: &'static str = "CREATE EXTENSION IF NOT EXISTS vector";
pub const HYBRID_SEARCH_RATIO: usize = 5;

pub fn create_search_base_sql() -> String {
    let query = format!(
        "CREATE TABLE IF NOT EXISTS {SEARCH_TABLES} (id bigserial PRIMARY KEY, table_name TEXT UNIQUE)"
    );
    return query;
}

pub fn create_rrf_score_function_sql() -> String {
    format!(
        "
        CREATE OR REPLACE FUNCTION rrf_score(rank bigint, rrf_k int DEFAULT 50)
        RETURNS numeric
        LANGUAGE SQL
        IMMUTABLE PARALLEL SAFE
        AS $$
            SELECT COALESCE(1.0/($1 + $2), 0.0);
        $$;
        "
    )
}

pub fn create_full_text_search_index(table_name: &str, column_name: &str) -> String {
    format!("CREATE INDEX IF NOT EXISTS {table_name}_{column_name}_idx ON {table_name} USING GIN (to_tsvector('english', {column_name}));")
}

pub fn create_vector_search_index(table_name: &str, vector_column_name: &str) -> String {
    format!(
        "CREATE INDEX IF NOT EXISTS {table_name}_{vector_column_name}_idx ON {table_name} USING  hnsw(({vector_column_name}::vector({EMBEDDING_DIMENSION})) vector_cosine_ops);"
    )
}

pub fn get_adj_chunk_sql(table_name: &str) -> String {
    format!(
        "SELECT *, CAST(0 AS FLOAT8) as score FROM {table_name} WHERE content_id = $1 AND chunk_number >= $2 AND chunk_number <= $3 ORDER BY chunk_number ASC"
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

pub fn keyword_search_from_table_sql(table_name: &str, max_similar_res: usize) -> String {
    format!("SELECT *, ts_rank_cd(to_tsvector('english', content_chunk), to_tsquery('english', '$1')) as score FROM {table_name} WHERE to_tsvector('english', content_chunk) @@ to_tsquery('english', '$1') ORDER BY rank DESC LIMIT {max_similar_res}")
}

pub fn semantic_search_from_table_sql(
    table_name: &str,
    max_similar_res: usize,
    minimum_score: f32,
) -> String {
    format!("SELECT *, CAST((1.0 - (content_embedding <=> $1::vector)) AS float4) AS score FROM {table_name} WHERE
    CAST((1.0-(content_embedding <=> $1::vector)) AS float4) >= {minimum_score} ORDER BY score DESC LIMIT {max_similar_res}")
}

pub fn hybrid_saerch_from_table_sql(table_name: &str, max_similar_res: usize) -> String {
    let max_hybrid_res = max_similar_res * HYBRID_SEARCH_RATIO;

    format!("
    WITH scored_results AS (
        SELECT {table_name}.*, CAST(sum(rrf_score({table_name}.rank)) OVER (PARTITION BY {table_name}.id) AS float4) AS score
        FROM (
            (
                SELECT *, rank() OVER (ORDER BY $1 <=> content_embedding) AS rank FROM {table_name}
                ORDER BY $1 <=> content_embedding LIMIT {max_hybrid_res}
            )
            UNION ALL
            (
                SELECT *, rank() OVER (ORDER BY ts_rank_cd(to_tsvector(content_chunk), plainto_tsquery($2)) DESC) AS rank
                FROM {table_name}
                WHERE
                    plainto_tsquery('english', $2) @@ to_tsvector('english', content_chunk)
                ORDER BY rank
                LIMIT {max_hybrid_res}
            )
        ) {table_name}
    ),
    ranked_results AS (
        SELECT *, ROW_NUMBER() OVER (PARTITION BY id ORDER BY score DESC) as row_num
        FROM scored_results    
    )
    SELECT *
    FROM ranked_results
    WHERE row_num = 1
    ORDER BY score DESC
    LIMIT {max_similar_res}
    ")
}
