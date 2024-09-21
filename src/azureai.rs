use crate::utils::vars;
use anyhow::Result;
use async_openai::types::ChatCompletionRequestMessage;
use async_openai::{config::AzureConfig, types::CreateChatCompletionRequestArgs, Client};
use futures::stream::SplitSink;
use futures::SinkExt;
use futures::StreamExt;

pub struct AzureAI<'a> {
    resource_name: &'a str,
    api_version: &'a str,
    deployment_id: &'a str,
    inf_model: &'a str,
    max_token: u16,
}

impl<'a> AzureAI<'a> {
    pub fn new(
        resource_name: &'a str,
        api_version: &'a str,
        deployment_id: &'a str,
        inf_model: &'a str,
        max_token: u16,
    ) -> Self {
        Self {
            resource_name,
            api_version,
            deployment_id,
            inf_model,
            max_token,
        }
    }
    pub async fn run_azureai(
        &self,
        websocket: &mut SplitSink<warp::ws::WebSocket, warp::filters::ws::Message>,
        messages: &Vec<ChatCompletionRequestMessage>,
    ) -> Result<String> {

        let auzre_ai_url = format!("https://{}.openai.azure.com", self.resource_name);

        let config = AzureConfig::new()
            .with_api_base(auzre_ai_url)
            .with_api_key(vars::get_azureai_api_key())
            .with_deployment_id(self.deployment_id)
            .with_api_version(self.api_version);

        let client = Client::with_config(config);

        let request = CreateChatCompletionRequestArgs::default()
            .model(self.inf_model)
            .max_tokens(self.max_token)
            .messages(messages.clone())
            .build()?;

        let mut stream = client.chat().create_stream(request).await?;

        let mut output: String = String::new();

        while let Some(res) = stream.next().await {
            match res {
                Ok(response) => {
                    if let Some(model_resp) = response.choices.first() {
                        if let Some(ref content) = model_resp.delta.content {
                            websocket.send(warp::ws::Message::text(content.clone()))
                            .await?;
                            output += content;
                        }
                    }
                }
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
            }
        }

        Ok(output)
    }
}
