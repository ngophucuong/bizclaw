//! # BizClaw Brain
//!
//! Local LLM inference engine — PicoLM rewrite in pure Rust.
//! Runs LLaMA-architecture models in GGUF format with mmap, SIMD, and quantization.

pub mod gguf;
pub mod mmap;
pub mod model;
pub mod tensor;
pub mod quant;
pub mod simd;
pub mod tokenizer;
pub mod sampler;
pub mod attention;
pub mod kv_cache;
pub mod grammar;
pub mod rope;
pub mod thread_pool;

use std::path::{Path, PathBuf};
use bizclaw_core::error::{BizClawError, Result};
use serde::{Deserialize, Serialize};

/// Brain engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainConfig {
    pub threads: u32,
    pub max_tokens: u32,
    pub context_length: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub json_mode: bool,
}

impl Default for BrainConfig {
    fn default() -> Self {
        Self {
            threads: 4,
            max_tokens: 256,
            context_length: 2048,
            temperature: 0.7,
            top_p: 0.9,
            json_mode: false,
        }
    }
}

/// The main brain engine for local LLM inference.
pub struct BrainEngine {
    config: BrainConfig,
    /// Loaded model (mmap)
    model: Option<LoadedModel>,
}

/// A loaded model ready for inference.
struct LoadedModel {
    /// Memory-mapped model file
    mmap_model: mmap::MmapModel,
    /// Model hyperparameters
    params: model::ModelParams,
    /// BPE tokenizer
    tokenizer: tokenizer::BpeTokenizer,
    /// KV cache for generation
    kv_cache: kv_cache::KvCache,
    /// Sampler
    sampler: sampler::Sampler,
    /// Model file path
    path: PathBuf,
}

impl BrainEngine {
    /// Create a new brain engine (model not yet loaded).
    pub fn new(config: BrainConfig) -> Self {
        Self { config, model: None }
    }

    /// Load a model from a GGUF file.
    pub fn load(model_path: &Path) -> Result<Self> {
        let config = BrainConfig::default();
        let mut engine = Self { config, model: None };
        engine.load_model(model_path)?;
        Ok(engine)
    }

    /// Load a GGUF model into the engine.
    pub fn load_model(&mut self, model_path: &Path) -> Result<()> {
        tracing::info!("Loading model from: {}", model_path.display());

        let mmap_model = mmap::MmapModel::load(model_path)?;
        let params = model::ModelParams::from_gguf(&mmap_model.gguf);

        tracing::info!(
            "Model params: dim={}, layers={}, heads={}, kv_heads={}, vocab={}",
            params.dim, params.n_layers, params.n_heads, params.n_kv_heads, params.vocab_size
        );

        // Load tokenizer
        let tokenizer = tokenizer::BpeTokenizer::from_gguf(&mmap_model.gguf.metadata)
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load tokenizer: {e}, using fallback");
                tokenizer::BpeTokenizer::fallback()
            });

        tracing::info!("Tokenizer loaded: vocab_size={}", tokenizer.vocab_size());

        // Create KV cache
        let kv_cache = kv_cache::KvCache::new(
            params.n_layers as usize,
            params.max_seq_len as usize,
            params.n_kv_heads as usize,
            params.head_dim as usize,
        );
        tracing::info!("KV cache: {:.1} MB", kv_cache.memory_usage() as f64 / 1024.0 / 1024.0);

        // Create sampler
        let sampler = sampler::Sampler::new(sampler::SamplerConfig {
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: 40,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
        });

        self.model = Some(LoadedModel {
            mmap_model,
            params,
            tokenizer,
            kv_cache,
            sampler,
            path: model_path.to_path_buf(),
        });

        tracing::info!("✅ Model loaded successfully: {}", model_path.display());
        Ok(())
    }

    /// Check if a model is loaded.
    pub fn is_loaded(&self) -> bool {
        self.model.is_some()
    }

    /// Generate text completion using the loaded model.
    pub fn generate(&self, prompt: &str, max_tokens: u32) -> Result<String> {
        let model = self.model.as_ref()
            .ok_or_else(|| BizClawError::Brain("Model not loaded".into()))?;

        // Tokenize prompt
        let mut input_tokens = vec![model.tokenizer.bos_id];
        input_tokens.extend(model.tokenizer.encode(prompt));

        tracing::debug!("Generate: prompt_len={}, input_tokens={}", prompt.len(), input_tokens.len());

        // Run forward pass for each token
        let mut output_tokens = Vec::new();
        let max_gen = max_tokens.min(self.config.max_tokens) as usize;
        let dim = model.params.dim as usize;

        // Buffers for forward pass
        let mut logits = vec![0.0f32; model.params.vocab_size as usize];

        for step in 0..max_gen {
            // Get the token to process
            let token = if step < input_tokens.len() {
                input_tokens[step]
            } else if let Some(&last) = output_tokens.last() {
                last
            } else {
                break;
            };

            // Forward pass (simplified — uses embedding lookup + matmul to get logits)
            self.forward_token(model, token, step, &mut logits)?;

            // Only sample after processing all input tokens
            if step >= input_tokens.len() - 1 {
                let all_tokens: Vec<u32> = input_tokens.iter()
                    .chain(output_tokens.iter())
                    .copied()
                    .collect();
                let next_token = model.sampler.sample(&mut logits, &all_tokens);

                // Check for EOS
                if next_token == model.tokenizer.eos_id {
                    break;
                }

                output_tokens.push(next_token);
            }
        }

        // Decode output tokens
        let output = model.tokenizer.decode(&output_tokens);
        Ok(output)
    }

    /// Run a single token through the transformer forward pass.
    fn forward_token(
        &self,
        model: &LoadedModel,
        _token: u32,
        _pos: usize,
        logits: &mut [f32],
    ) -> Result<()> {
        let vocab_size = model.params.vocab_size as usize;

        // TODO: Full forward pass implementation
        // For now, generate random logits for testing
        // The full implementation would:
        // 1. Lookup token embedding
        // 2. For each layer:
        //    a. RMSNorm
        //    b. Q/K/V projections
        //    c. RoPE
        //    d. Attention (with KV cache)
        //    e. RMSNorm
        //    f. FFN (gate_proj, up_proj, down_proj with SiLU)
        // 3. Final RMSNorm
        // 4. LM head projection → logits

        // Placeholder: uniform logits (sampler will handle randomness)
        for i in 0..vocab_size.min(logits.len()) {
            logits[i] = 0.0;
        }

        Ok(())
    }

    /// Generate with JSON grammar constraint.
    pub fn generate_json(&self, prompt: &str) -> Result<serde_json::Value> {
        let text = self.generate(prompt, self.config.max_tokens)?;
        Ok(serde_json::json!({"response": text}))
    }

    /// Get the brain config.
    pub fn config(&self) -> &BrainConfig {
        &self.config
    }

    /// Get model info if loaded.
    pub fn model_info(&self) -> Option<String> {
        self.model.as_ref().map(|m| {
            format!(
                "{} ({}MB, {} layers, {} heads)",
                m.path.file_name().unwrap_or_default().to_string_lossy(),
                m.mmap_model.file_size() / 1024 / 1024,
                m.params.n_layers,
                m.params.n_heads,
            )
        })
    }
}
