use crate::utils::utils;
use crate::utils::utils::get_device;
use crate::utils::vars;
use anyhow::{Context, Error, Result};
use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use lazy_static::lazy_static;
use tokenizers::{PaddingParams, Tokenizer};

lazy_static! {
    pub static ref EMB_MODEL: (BertModel, Tokenizer) = load_model().expect("Failed to Load Model");
}

pub fn load_model() -> Result<(BertModel, Tokenizer)> {
    let embedding_model_path = vars::get_embedding_model_path();

    let config_path = embedding_model_path.join("config.json");
    let tokenizer_path = embedding_model_path.join("tokenizer.json");
    let weights_path = embedding_model_path.join("pytorch_model.bin");
    let config = std::fs::read_to_string(&config_path)?;

    let config: Config = serde_json::from_str(&config)?;

    let mut tokenizer = Tokenizer::from_file(tokenizer_path).map_err(Error::msg)?;

    let vb = VarBuilder::from_pth(&weights_path, DTYPE, &utils::get_device())?;

    let model = BertModel::load(vb, &config)?;

    if let Some(pp) = tokenizer.get_padding_mut() {
        pp.strategy = tokenizers::PaddingStrategy::BatchLongest
    } else {
        let pp = PaddingParams {
            strategy: tokenizers::PaddingStrategy::BatchLongest,
            ..Default::default()
        };
        tokenizer.with_padding(Some(pp));
    }

    Ok((model, tokenizer))
}

pub fn get_embeddings(sentence: &str) -> Result<Tensor> {
    let (model, tokenizer) = &*EMB_MODEL;

    let sentence: String = sentence.chars().filter(|c| c.is_ascii()).collect();

    let tokens = tokenizer
        .encode_batch(vec![sentence], true)
        .map_err(Error::msg)?;

    let token_ids = tokens
        .iter()
        .map(|tokens| {
            let tokens = tokens.get_ids().to_vec();
            Ok(Tensor::new(tokens.as_slice(), &get_device())?)
        })
        .collect::<Result<Vec<_>>>()
        .context("Unable to get token ids")?;

    let token_ids = Tensor::stack(&token_ids, 0).context("Unable to stack token ids")?;

    let token_type_ids = token_ids.zeros_like().context("Unable to get embeddings")?;

    let embeddings = model.forward(&token_ids, &token_type_ids)?;

    let (_n_sentence, n_tokens, _hidden_size) = embeddings
        .dims3()
        .context("Unable to get embeddings dimensions")?;

    let embeddings: Tensor =
        (embeddings.sum(1)? / (n_tokens as f64)).context("Unable to get Embedding sum")?;

    let embeddings = embeddings.broadcast_div(&embeddings.sqr()?.sum_keepdim(1)?.sqrt()?)?;

    Ok(embeddings)
}
