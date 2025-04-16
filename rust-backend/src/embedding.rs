// src/embedding.rs
use anyhow::{anyhow, Result};
use candle_core::{DType, Device, Tensor};
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use log::{debug, info, warn};
use std::path::Path;
use std::sync::Arc;
use tokenizers::Tokenizer;

const MODEL_PATH: &str = "./models/bge-small-en-v1.5/model.safetensors";
const TOKENIZER_PATH: &str = "./models/bge-small-en-v1.5/tokenizer.json";
const CONFIG_PATH: &str = "./models/bge-small-en-v1.5/config.json";

pub struct EmbeddingModel {
    model: Arc<BertModel>,
    tokenizer: Arc<Tokenizer>,
    device: Arc<Device>,
}

pub fn init_model() -> Result<EmbeddingModel> {
    // Check if model files exist
    if !Path::new(MODEL_PATH).exists() {
        return Err(anyhow!(
            "Model file not found at {}. Please download the model files.",
            MODEL_PATH
        ));
    }

    if !Path::new(TOKENIZER_PATH).exists() {
        return Err(anyhow!(
            "Tokenizer file not found at {}. Please download the model files.",
            TOKENIZER_PATH
        ));
    }

    if !Path::new(CONFIG_PATH).exists() {
        return Err(anyhow!(
            "Config file not found at {}. Please download the model files.",
            CONFIG_PATH
        ));
    }

    info!("Loading tokenizer from {}", TOKENIZER_PATH);
    let tokenizer = Tokenizer::from_file(TOKENIZER_PATH)
        .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;
    info!("Tokenizer loaded successfully");

    // Try to use Metal for Mac, fall back to CPU
    info!("Attempting to initialize Metal device");
    let device = match Device::new_metal(0) {
        Ok(device) => {
            info!("Successfully initialized Metal device for GPU acceleration");
            device
        }
        Err(e) => {
            warn!("Metal not available, falling back to CPU: {}", e);
            Device::Cpu
        }
    };
    info!("Using device: {:?}", device);

    // Load weights
    info!("Loading model weights from {}", MODEL_PATH);
    let weights = candle_core::safetensors::load(MODEL_PATH, &device)
        .map_err(|e| anyhow!("Failed to load model weights: {}", e))?;
    info!("Model weights loaded successfully");

    // Load configuration
    let config_contents = std::fs::read_to_string(CONFIG_PATH)
        .map_err(|e| anyhow!("Failed to read config file: {}", e))?;
    
    let model_config: BertConfig = serde_json::from_str(&config_contents)
        .map_err(|e| anyhow!("Failed to parse config JSON: {}", e))?;
    info!("Model configuration loaded successfully");

    // Initialize model
    info!("Initializing model from weights");
    let vb = candle_nn::VarBuilder::from_tensors(weights, DType::F32, &device);
    
    let model = BertModel::load(vb, &model_config)
        .map_err(|e| anyhow!("Failed to initialize model: {}", e))?;
    info!("Model initialized successfully");

    Ok(EmbeddingModel {
        model: Arc::new(model),
        tokenizer: Arc::new(tokenizer),
        device: Arc::new(device),
    })
}

pub async fn generate_embedding(text: &str, model: &EmbeddingModel) -> Result<Vec<f32>> {
    debug!("Generating embedding for text: '{}'", text);
    
    // Tokenize the input text
    let encoding = model.tokenizer.encode(text, true)
        .map_err(|e| anyhow!("Failed to encode text: {}", e))?;
    
    let input_ids = encoding.get_ids().to_vec();
    let attention_mask = encoding.get_attention_mask().to_vec();
    
    // Calculate mask_sum before moving attention_mask
    let mask_sum = attention_mask.iter().sum::<u32>() as usize;
    if mask_sum == 0 {
        return Err(anyhow!("Invalid input: empty attention mask"));
    }
    
    // Convert to tensors
    let input_ids_len = input_ids.len();
    let attention_mask_len = attention_mask.len();
    
    let input_ids_tensor = Tensor::from_vec(input_ids, (1, input_ids_len), &model.device)
        .map_err(|e| anyhow!("Failed to create input_ids tensor: {}", e))?;
    
    let attention_mask_tensor = Tensor::from_vec(attention_mask, (1, attention_mask_len), &model.device)
        .map_err(|e| anyhow!("Failed to create attention_mask tensor: {}", e))?;
    
    // Get embeddings from model
    let embeddings = model.model.forward(&input_ids_tensor, &attention_mask_tensor, None)
        .map_err(|e| anyhow!("Model forward pass failed: {}", e))?;
    
    // Mean pooling - take average of all token embeddings
    let embedding_dim = embeddings.dims()[2];
    let mut pooled_embedding = vec![0.0; embedding_dim];
    
    // Extract the embeddings
    for j in 0..mask_sum {
        let token_embedding: Vec<f32> = embeddings
            .get(0)?
            .get(j)?
            .to_vec1()?;
        
        for k in 0..embedding_dim {
            pooled_embedding[k] += token_embedding[k];
        }
    }
    
    // Average the embeddings
    for k in 0..embedding_dim {
        pooled_embedding[k] /= mask_sum as f32;
    }
    
    // Normalize the embedding to unit length (very important for cosine similarity)
    let norm: f32 = pooled_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    for k in 0..embedding_dim {
        pooled_embedding[k] /= norm;
    }
    
    debug!("Embedding generation successful, vector dimension: {}", embedding_dim);
    Ok(pooled_embedding)
}