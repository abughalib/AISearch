#[cfg(test)]
mod sql_query_test {
    use crate::constants::*;
    pub const TABLE_NAME: &'static str = "test_table";

    #[test]
    fn test_create_search_base_sql() {
        assert_eq!(create_search_base_sql(), format!("CREATE TABLE IF NOT EXISTS {SEARCH_TABLES} (id bigserial PRIMARY KEY, table_name TEXT UNIQUE)"))
    }

    #[test]
    fn test_get_adj_chunk_sql() {
        assert_eq!(get_adj_chunk_sql(TABLE_NAME), "SELECT * FROM test_table WHERE content_id = $1 AND chunk_number >= $2 AND chunk_number <= $3 ORDER BY chunk_number ASC")
    }

    #[test]
    fn test_create_vector_table_sql() {
        assert_eq!(
            create_vector_table_sql(TABLE_NAME),
            "CREATE TABLE IF NOT EXISTS test_table (id bigserial PRIMARY KEY, content_id TEXT, content_chunk TEXT, chunk_number int, embedding vector(1024), metadata JSON, created_at timestamp)"
        )
    }

    #[test]
    fn test_create_raw_content_table_sql() {
        assert_eq!(
            create_raw_content_table_sql(TABLE_NAME),
            "CREATE TABLE IF NOT EXISTS test_table_content (id bigserial PRIMARY KEY, content_id TEXT, title TEXT, text TEXT, metadata JSON)"
        )
    }

    #[test]
    fn test_insert_into_search_table_sql() {
        assert_eq!(
            insert_into_search_table_sql(),
            format!("INSERT INTO {SEARCH_TABLES} (table_name) VALUES ($1)")
        )
    }

    #[test]
    fn test_bulk_insert_into_vector_table_sql() {
        assert_eq!(
            bulk_insert_into_vector_table_sql(TABLE_NAME).replace("\n", "").replace("   ", "").trim(),
            format!("
                INSERT INTO {TABLE_NAME}(content_id, content_chunk, chunk_number, embedding, metadata, created_at) 
                SELECT * FROM UNNEST($1::text[], $2::text[], $3::int4[], $4::vector[], $5::jsonb[], $6::timestamp[])
            ").replace("\n", "").replace("   ", "").trim()
        )
    }

    #[test]
    fn test_insert_raw_content_sql() {
        assert_eq!(insert_raw_content_sql(TABLE_NAME), "INSERT INTO test_table_content (content_id, title, text, metadata) VALUES ($1, $2, $3, $4::jsonb)")
    }

    #[test]
    fn test_get_similar_result_query() {
        assert_eq!(
            get_similar_result_query(TABLE_NAME, 10, 0.5),
            "SELECT *, (1.0-(embedding <=> $1::vector)) as score FROM test_table WHERE (1.0-(embedding <=> $1::vector)) >=0.5 ORDER BY score DESC LIMIT 10"
        )
    }

    #[test]
    fn test_get_search_tables_sql() {
        assert_eq!(
            get_search_tables_sql(),
            format!("SELECT table_name FROM {SEARCH_TABLES}")
        )
    }

    #[test]
    fn test_get_drop_table_sql() {
        assert_eq!(
            get_drop_table_sql(TABLE_NAME.to_string()),
            format!("DROP TABLE IF EXISTS {TABLE_NAME}")
        )
    }

    #[test]
    fn test_get_drop_search_tables_sql() {
        assert_eq!(
            get_delete_from_search_table_sql(TABLE_NAME),
            format!("DELETE FROM {SEARCH_TABLES} WHERE table_name = '{TABLE_NAME}'")
        )
    }
}
