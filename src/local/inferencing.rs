use core::fmt;
use std::collections::HashMap;

use crate::azureai::AzureAI;
use crate::local::database::EmbeddingVectorValue;
use crate::local::{database, embedding};
use crate::{vars, utils};
use anyhow::{Error as E, Result};
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestUserMessageContent,
};
use candle_core::Tensor;
use candle_core::{DType, Device};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_mixformer::Config;
use candle_transformers::models::quantized_mixformer::MixFormerSequentialForCausalLM as QMixFormer;
use futures::stream::SplitSink;
use futures::SinkExt;
use lazy_static::lazy_static;
use serde_json::json;
use tokenizers::Tokenizer;

const MAXIMUM_SAMPLE_LEN: usize = 512;

lazy_static! {
    pub static ref PHI: (QMixFormer, Tokenizer) = load_model().expect("Unable to Load Model");
}

const EMBEDDING_DIMENSION: usize = 1024;

pub fn load_model() -> Result<(QMixFormer, Tokenizer)> {
    let quantized_path = vars::get_inferencing_model_path();

    let tokerizer_file = quantized_path.join("tokenizer.json");

    let tokenizer =
        Tokenizer::from_file(tokerizer_file).expect("Failed to load Quantized tokenizer");

    let weights_filename = quantized_path.join("model-q4k.gguf");

    let config = Config::v2();

    let vb = candle_transformers::quantized_var_builder::VarBuilder::from_gguf(
        &weights_filename,
        &utils::get_device(),
    )?;

    let model = QMixFormer::new_v2(&config, vb).map_err(E::msg)?;

    Ok((model, tokenizer))
}

pub struct TextGeneration {
    pub model: QMixFormer,
    pub device: Device,
    pub tokenizer: Tokenizer,
    pub logits_processor: LogitsProcessor,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
}

impl TextGeneration {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        model: QMixFormer,
        tokenizer: Tokenizer,
        seed: u64,
        temp: Option<f64>,
        top_p: Option<f64>,
        repeat_penalty: f32,
        repeat_last_n: usize,
    ) -> Self {
        let logits_processor = LogitsProcessor::new(seed, temp, top_p);
        Self {
            model,
            device: utils::get_device(),
            tokenizer,
            logits_processor,
            repeat_penalty,
            repeat_last_n,
        }
    }

    pub async fn run(
        &mut self,
        prompt: &str,
        sample_len: usize,
        websocket: &mut SplitSink<warp::ws::WebSocket, warp::filters::ws::Message>,
    ) -> Result<String> {
        let mut response: String = String::new();

        let tokens = self.tokenizer.encode(prompt, true).map_err(E::msg)?;
        if tokens.is_empty() {
            anyhow::bail!("Empty prompts are not supported in the phi model.")
        }
        let mut tokens = tokens.get_ids().to_vec();
        let eos_token = match self.tokenizer.get_vocab(true).get("<|im_end|>") {
            Some(token) => *token,
            None => panic!("cannot find the endoftext token"),
        };
        for index in 0..sample_len {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let ctxt = &tokens[tokens.len().saturating_sub(context_size)..];
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input)?;
            let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
            let logits = if self.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repeat_penalty,
                    &tokens[start_at..],
                )?
            };

            let next_token = self.logits_processor.sample(&logits)?;
            tokens.push(next_token);
            if next_token == eos_token {
                break;
            }
            let token = self.tokenizer.decode(&[next_token], true).map_err(E::msg)?;

            websocket
                .send(warp::ws::Message::text(token.clone()))
                .await?;

            response += &token;
        }
        return Ok(response);
    }
}

pub enum MessageType {
    System,
    User,
    Assistant,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => {
                write!(f, "user")
            }
            Self::Assistant => {
                write!(f, "assistant")
            }
            Self::System => {
                write!(f, "system")
            }
        }
    }
}

pub struct ModelQuery {
    system_message: String,
    chat_history: HashMap<String, Vec<(MessageType, String)>>,
}

impl ModelQuery {
    pub fn new() -> Self {
        Self {
            system_message: format!(
                "You're a friendly and helpful AI Assistant. Be Concise and don't repeat yourself"
            ),
            chat_history: HashMap::new(),
        }
    }

    pub fn add_user_message(
        &mut self,
        question: &str,
        session_id: &str,
        references: &Vec<EmbeddingVectorValue>,
    ) {
        let mut context = Vec::new();

        for reference in references.clone() {
            context.push(json!({
                "content": reference.content_chunk,
                "metadata": reference.metadata
            }))
        }

        let context: String = json!(context).to_string();

        let next_message = format!("question: \"{question}\"\nreferences: \"{context}\"\n");

        match self.chat_history.get_mut(session_id) {
            Some(messages) => {
                messages.push((MessageType::User, next_message));
            }
            None => {
                self.chat_history.insert(
                    session_id.to_string(),
                    vec![
                        (MessageType::System, self.system_message.clone()),
                        (MessageType::User, next_message),
                    ],
                );
            }
        }
    }
    pub fn add_assistant_message(&mut self, session_id: &str, response: String) {
        match self.chat_history.get_mut(session_id) {
            Some(message) => message.push((MessageType::Assistant, response)),
            None => {
                self.chat_history.insert(
                    session_id.to_string(),
                    vec![(MessageType::Assistant, response)],
                );
            }
        }
    }
    pub fn build_local(&self, session_id: &str) -> String {
        match self.chat_history.get(session_id) {
            Some(history) => {
                let mut result = String::new();

                for (msg_type, message) in history {
                    match msg_type {
                        MessageType::System => {
                            result += &format!("<|im_start|>system\n{message}<|im_end|>\n")
                        }
                        MessageType::User => {
                            result += &format!("<|im_start|>user\n{message}<|im_end|>\n")
                        }
                        MessageType::Assistant => {
                            result += &format!("<|im_start|>assistant\n{message}<|im_end|>\n")
                        }
                    }
                }

                return result;
            }
            None => {
                println!("No History");
                return String::new();
            }
        }
    }
    pub fn build_openai(&self, session_id: &str) -> Result<Vec<ChatCompletionRequestMessage>> {
        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();

        match self.chat_history.get(session_id) {
            Some(history) => {
                for (msg_type, message) in history {
                    match msg_type {
                        MessageType::System => messages.push(
                            ChatCompletionRequestSystemMessageArgs::default()
                                .content(message)
                                .build()?
                                .into(),
                        ),
                        MessageType::User => messages.push(
                            ChatCompletionRequestUserMessageArgs::default()
                                .content(ChatCompletionRequestUserMessageContent::Text(
                                    message.to_owned(),
                                ))
                                .build()?
                                .into(),
                        ),
                        MessageType::Assistant => messages.push(
                            ChatCompletionRequestAssistantMessageArgs::default()
                                .content(message)
                                .build()?
                                .into(),
                        ),
                    }
                }
            }
            None => {
                println!("No History!");
            }
        }

        return Ok(messages);
    }
    pub async fn answer_with_context(
        &mut self,
        websocket: &mut SplitSink<warp::ws::WebSocket, warp::filters::ws::Message>,
        query: &str,
        session_id: &str,
        deployment_type: &str,
        deployment_model: &str,
        references: Vec<EmbeddingVectorValue>,
    ) -> Result<()> {
        if references.is_empty() {
            websocket
                .send(warp::ws::Message::text(
                    "The query doesn't match any refences",
                ))
                .await?;

            return Ok(());
        }

        match deployment_type {
            "AZURE" => {
                let azure_conn =
                    AzureAI::new("abu-openai", "2024-02-01", "gpt-4o", deployment_model, 2048);

                self.add_user_message(query, session_id, &references);

                let resp = azure_conn
                    .run_azureai(websocket, &self.build_openai(session_id)?)
                    .await?;

                let _ = websocket
                    .send(warp::ws::Message::text("Reference Source: \n"))
                    .await;

                for source in references.iter() {
                    let _ = websocket
                        .send(warp::ws::Message::text(format!(
                            "{}",
                            source.metadata["source"]
                                .to_string()
                                .split(".")
                                .nth(0)
                                .unwrap_or("")
                        )))
                        .await?;
                }

                self.add_assistant_message(session_id, resp);
            }
            _ => {
                let (model, tokenizer) = &*PHI;

                let mut pipeline = TextGeneration::new(
                    model.clone(),
                    tokenizer.clone(),
                    12345,
                    Some(0.7),
                    None,
                    1.1,
                    64,
                );

                self.add_user_message(query, session_id, &references);

                let resp = pipeline
                    .run(&self.build_local(session_id), MAXIMUM_SAMPLE_LEN, websocket)
                    .await?;

                let _ = websocket
                    .send(warp::ws::Message::text("Reference Source: \n"))
                    .await?;

                for source in references.iter() {
                    let _ = websocket
                        .send(warp::ws::Message::text(format!(
                            "{}",
                            source.metadata["source"]
                                .to_string()
                                .split(".")
                                .nth(0)
                                .unwrap_or("")
                        )))
                        .await?;
                }

                self.add_assistant_message(session_id, resp);
            }
        }

        Ok(())
    }
    pub async fn get_embeddings(
        &self,
        table_name: &str,
        query: &str,
        max_similar_res: usize,
        lower_chunk: i32,
        upper_chunk: i32,
        minimum_score: f32,
    ) -> Result<Vec<EmbeddingVectorValue>> {
        let embeddings: Vec<f32> = embedding::get_embeddings(&query)?
            .reshape((EMBEDDING_DIMENSION,))?
            .to_vec1()?;

        let references = database::get_similar_results(
            table_name,
            embeddings.into(),
            max_similar_res,
            minimum_score,
        )
        .await?;

        let mut final_ref: Vec<EmbeddingVectorValue> = Vec::with_capacity(references.len());

        for (i, reference) in references.iter().enumerate() {
            let related = reference
                .get_adjacent_chunks(table_name, upper_chunk, lower_chunk)
                .await?;

            let mut chunks: String = String::new();

            for r in related.iter() {
                chunks += &r.content_chunk;
                chunks.push(' ');
            }

            final_ref.push(EmbeddingVectorValue {
                id: i as i64,
                content_id: reference.content_id.to_owned(),
                content_chunk: chunks,
                chunk_number: reference.chunk_number,
                embedding: reference.embedding.clone(),
                metadata: reference.metadata.clone(),
                create_at: reference.create_at,
                score: reference.score,
            });
        }

        return Ok(final_ref);
    }
}
