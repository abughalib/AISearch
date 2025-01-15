use crate::utils::vars;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig<'a> {
    pub app_config: AppSettings<'a>,
    pub web_config: WebSettings<'a>,
    pub azure_config_llm_inferencing: AzureConfigLLMInferencing<'a>,
    pub azure_config_slm_inferencing: AzureConfigSLMInferencing<'a>,
    pub azure_embedding_config: AzureEmbeddingConfig<'a>,
    pub local_embedding_config: LocalEmbeddingConfig<'a>,
    pub database_config: DatabaseConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppSettings<'a> {
    pub maximum_upload_size: u32,
    pub embedding_model: Cow<'a, str>,
    pub inferencing_model: Cow<'a, str>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebSettings<'a> {
    pub ip_address: Cow<'a, str>,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AzureConfigLLMInferencing<'a> {
    pub resource_name: Cow<'a, str>,
    pub api_version: Cow<'a, str>,
    pub deployment_id: Cow<'a, str>,
    pub inferencing_model: Cow<'a, str>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AzureConfigSLMInferencing<'a> {
    pub resource_name: Cow<'a, str>,
    pub api_version: Cow<'a, str>,
    pub deployment_id: Cow<'a, str>,
    pub inferencing_model: Cow<'a, str>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AzureEmbeddingConfig<'a> {
    pub resource_name: Cow<'a, str>,
    pub api_version: Cow<'a, str>,
    pub deployment_id: Cow<'a, str>,
    pub embedding_model: Cow<'a, str>,
    pub dimension: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalEmbeddingConfig<'a> {
    pub embedding_model: Cow<'a, str>,
    pub dimension: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub search_type: SearchType,
}

#[derive(Debug, Deserialize)]
pub enum SearchType {
    #[serde(rename = "semantic_search")]
    SemanticSearch,
    #[serde(rename = "keyword_search")]
    KeyWordSearch,
    #[serde(rename = "hybrid_search")]
    HybridSearch,
}

impl ToString for SearchType {
    fn to_string(&self) -> String {
        match self {
            SearchType::SemanticSearch => "semantic_search".to_string(),
            SearchType::KeyWordSearch => "keyword_search".to_string(),
            SearchType::HybridSearch => "hybrid_search".to_string(),
        }
    }
}

impl Serialize for SearchType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl std::str::FromStr for SearchType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "semantic_search" => Ok(SearchType::SemanticSearch),
            "keyword_search" => Ok(SearchType::KeyWordSearch),
            "hybrid_search" => Ok(SearchType::HybridSearch),
            _ => Err(format!("Invalid search type: {}", s)),
        }
    }
}

impl<'a> AppConfig<'a> {
    pub fn default() -> Self {
        Self {
            app_config: AppSettings {
                maximum_upload_size: 100,
                embedding_model: Cow::Borrowed("local"),
                inferencing_model: Cow::Borrowed("local"),
            },
            web_config: WebSettings {
                ip_address: Cow::Borrowed("127.0.0.1"),
                port: 8000,
            },
            azure_config_llm_inferencing: AzureConfigLLMInferencing {
                resource_name: Cow::Borrowed("openai"),
                api_version: Cow::Borrowed("2022-12-01"),
                deployment_id: Cow::Borrowed("text-embedding-ada-002"),
                inferencing_model: Cow::Borrowed("text-embedding-ada-002"),
            },
            azure_config_slm_inferencing: AzureConfigSLMInferencing {
                resource_name: Cow::Borrowed("openai"),
                api_version: Cow::Borrowed("2022-12-01"),
                deployment_id: Cow::Borrowed("text-search-ada-doc-001"),
                inferencing_model: Cow::Borrowed("text-search-ada-doc-001"),
            },
            local_embedding_config: LocalEmbeddingConfig {
                embedding_model: Cow::Borrowed("BAAI_V1.5L"),
                dimension: 1024,
            },
            azure_embedding_config: AzureEmbeddingConfig {
                resource_name: Cow::Borrowed("openai"),
                api_version: Cow::Borrowed("2022-12-01"),
                deployment_id: Cow::Borrowed("text-embedding-ada-002"),
                embedding_model: Cow::Borrowed("text-embedding-ada-002"),
                dimension: None,
            },
            database_config: DatabaseConfig {
                search_type: SearchType::SemanticSearch,
            },
        }
    }
    pub fn save(&self) -> std::io::Result<()> {
        let config = toml::to_string(&self).unwrap();
        std::fs::write(vars::get_app_config_path(), config)
    }
    pub fn load() -> Result<Self> {
        match std::fs::read_to_string(vars::get_app_config_path()) {
            Ok(config) => Ok(toml::from_str(&config)?),
            Err(_) => Ok(AppConfig::default()),
        }
    }
}
