use crate::utils::vars;
use anyhow::Result;
use async_openai::{
    config::AzureConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
    Client,
};
use futures::SinkExt;
use futures::{stream::SplitSink, StreamExt};

pub struct AzureInferencing<'a> {
    resource_name: &'a str,
    api_version: &'a str,
    deployment_id: &'a str,
    inf_model: &'a str,
    max_token: u16,
    seed: u32,
    temperature: f32,
    top_p: f32,
    frequency_penalty: f32,
}

impl<'a> AzureInferencing<'a> {
    pub fn default() -> Self {
        Self {
            resource_name: "",
            api_version: "",
            deployment_id: "",
            inf_model: "",
            max_token: 2048,
            seed: 12345,
            temperature: 0.6,
            top_p: 1.1,
            frequency_penalty: 0.0,
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
    pub fn with_inferencing_model(mut self, inf_model: &'a str) -> Self {
        self.inf_model = inf_model;
        self
    }

    pub fn with_max_token(mut self, max_token: u16) -> Self {
        self.max_token = max_token;
        self
    }

    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p;
        self
    }

    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = frequency_penalty;
        self
    }
    pub async fn chat(&self, messages: &Vec<ChatCompletionRequestMessage>) -> Result<String> {
        let client = Client::with_config(
            AzureConfig::new()
                .with_api_base(format!("https://{}.openai.azure.com/", self.resource_name))
                .with_api_version(self.api_version)
                .with_deployment_id(self.deployment_id)
                .with_api_key(&vars::get_azureai_api_key()),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(self.inf_model)
            .messages(messages.clone())
            .max_tokens(self.max_token)
            .seed(self.seed)
            .temperature(self.temperature)
            .top_p(self.top_p)
            .frequency_penalty(self.frequency_penalty)
            .build()?;

        let response = client.chat().create(request).await?;

        return Ok(response.choices[0].message.content.clone().unwrap());
    }

    pub async fn stream(
        &self,
        messages: &Vec<ChatCompletionRequestMessage>,
        websocket: &mut SplitSink<warp::ws::WebSocket, warp::filters::ws::Message>,
    ) -> Result<String> {
        let client = Client::with_config(
            AzureConfig::new()
                .with_api_base(format!("https://{}.openai.azure.com/", self.resource_name))
                .with_api_version(self.api_version)
                .with_deployment_id(self.deployment_id)
                .with_api_key(&vars::get_azureai_api_key()),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(self.inf_model)
            .messages(messages.clone())
            .max_tokens(self.max_token)
            .seed(self.seed)
            .temperature(self.temperature)
            .top_p(self.top_p)
            .frequency_penalty(self.frequency_penalty)
            .build()?;

        let mut response = client.chat().create_stream(request).await?;

        let mut output: String = String::new();

        while let Some(result) = response.next().await {
            match result {
                Ok(response) => {
                    if let Some(content) = response.choices.first() {
                        if let Some(ref content) = content.delta.content {
                            output.push_str(content);
                            websocket
                                .send(warp::ws::Message::text(output.clone()))
                                .await?;
                        }
                    }
                }
                Err(_e) => {
                    break;
                }
            }
        }
        Ok(output)
    }
}
