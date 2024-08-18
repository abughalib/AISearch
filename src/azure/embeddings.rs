use crate::vars;
use anyhow::Result;
use async_openai::types::CreateEmbeddingRequestArgs;
use async_openai::Client;
use langchain_rust::llm::AzureConfig;

pub struct AzureEmbedding<'a> {
    resource_name: &'a str,
    api_version: &'a str,
    deployment_id: &'a str,
    embed_model: &'a str,
}

impl<'a> AzureEmbedding<'a> {
    pub fn default() -> Self {
        Self {
            resource_name: "",
            api_version: "",
            deployment_id: "",
            embed_model: "",
        }
    }
    pub fn with_resource_name(mut self, resource_name: &'a str) -> Self {
        self.resource_name = resource_name;
        self
    }
    pub fn with_api_version(mut self, api_version: &'a str) -> Self {
        self.api_version = api_version;
        self
    }
    pub fn with_deployment_id(mut self, deployment_id: &'a str) -> Self {
        self.deployment_id = deployment_id;
        self
    }
    pub fn with_embedding_model(mut self, embed_model: &'a str) -> Self {
        self.embed_model = embed_model;
        self
    }

    pub async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        let client = Client::with_config(
            AzureConfig::new()
                .with_api_base(format!("https://{}.openai.azure.com/", self.resource_name))
                .with_api_version(self.api_version)
                .with_deployment_id(self.deployment_id)
                .with_api_key(&vars::get_azureai_api_key()),
        );

        let request = CreateEmbeddingRequestArgs::default()
            .model(self.embed_model)
            .input(text)
            .build()?;

        let response = client.embeddings().create(request).await?;

        let embeddings = response.data;

        if let Some(embedding) = embeddings.first() {
            Ok(embedding.embedding.clone())
        } else {
            anyhow::bail!("Embedding generation failed.")
        }
    }
}
