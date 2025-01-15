#[cfg(test)]
mod azure_embedding_test {
    use crate::azure::embeddings::AzureEmbedding;

    #[ignore]
    #[tokio::test]
    async fn test_correct_embedding_generated() {
        let azure_embedding = AzureEmbedding::default()
            .with_resource_name("testing")
            .with_api_version("2024-02-01")
            .with_deployment_id("testembedding")
            .with_embedding_model("text-embedding-ada-002");

        let embedding = azure_embedding
            .generate_embedding("Some test string")
            .await
            .unwrap();

        assert_eq!(embedding.len(), 1536);
    }
}

#[cfg(test)]
mod azure_inferencing_tests {
    use crate::azure::inferencing::AzureInferencing;
    use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs};

    #[ignore]
    #[tokio::test]
    async fn test_correct_inference_generated() {
        let azure_inferencing = AzureInferencing::default()
            .with_resource_name("testing")
            .with_api_version("2024-02-01")
            .with_deployment_id("gpt-4o")
            .with_inferencing_model("gpt-4o-mini");

        let messages = ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessageArgs::default()
                .content("Test Message")
                .build()
                .unwrap(),
        );

        let completion = azure_inferencing.chat(&vec![messages]).await;

        assert!(completion.is_ok());
    }
}
