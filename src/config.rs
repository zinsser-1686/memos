//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub vault: VaultConfig,
    pub embedding: EmbeddingConfig,
    pub llm: LlmConfig,
    pub reranker: RerankerConfig,
    pub retrieval: RetrievalConfig,
    pub lifecycle: LifecycleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    pub path: String,
    pub relations_db: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub model: String,
    pub dimension: usize,
    pub local: LocalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub local: LocalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankerConfig {
    pub provider: String,
    pub model: String,
    pub local: LocalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub url: Option<String>,
    pub no_think: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    pub bm25: Bm25Config,
    pub vector: VectorConfig,
    pub rrf: RrfConfig,
    pub rerank: RerankConfig,
    pub mmr: MmrConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bm25Config {
    pub k1: f32,
    pub b: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    pub hnsw: HnswConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfig {
    pub m: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RrfConfig {
    pub k: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankConfig {
    pub context_chars: usize,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmrConfig {
    pub enabled: bool,
    pub lambda: f32,
    pub diversity_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleConfig {
    pub reflect: ReflectConfig,
    pub decay: DecayConfig,
    pub governance: GovernanceConfig,
    pub consolidation: ConsolidationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectConfig {
    pub enabled: bool,
    pub interval_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub max_memory: usize,
    pub per_type_limits: PerTypeLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerTypeLimits {
    pub identity: Option<usize>,
    pub emotion: Option<usize>,
    pub knowledge: Option<usize>,
    pub event: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    pub enabled: bool,
    pub deduplication_window_minutes: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vault: VaultConfig {
                path: "memos.mv2".to_string(),
                relations_db: "memos.db".to_string(),
            },
            embedding: EmbeddingConfig {
                provider: "local".to_string(),
                model: "BAAI/bge-small-en-v1.5".to_string(),
                dimension: 384,
                local: LocalConfig {
                    url: Some("http://localhost:8088".to_string()),
                    no_think: None,
                },
            },
            llm: LlmConfig {
                provider: "local".to_string(),
                model: "qwen3-1.7b-q4_k_m".to_string(),
                local: LocalConfig {
                    url: Some("http://localhost:8089".to_string()),
                    no_think: Some(true),
                },
            },
            reranker: RerankerConfig {
                provider: "local".to_string(),
                model: "cross-encoder/ms-marco-MiniLM-L-12-v2".to_string(),
                local: LocalConfig {
                    url: Some("http://localhost:8090".to_string()),
                    no_think: None,
                },
            },
            retrieval: RetrievalConfig {
                bm25: Bm25Config { k1: 1.5, b: 0.75 },
                vector: VectorConfig {
                    hnsw: HnswConfig {
                        m: 16,
                        ef_construction: 200,
                        ef_search: 100,
                    },
                },
                rrf: RrfConfig { k: 60 },
                rerank: RerankConfig {
                    context_chars: 4000,
                    batch_size: 8,
                },
                mmr: MmrConfig {
                    enabled: true,
                    lambda: 0.7,
                    diversity_threshold: 0.6,
                },
            },
            lifecycle: LifecycleConfig {
                reflect: ReflectConfig {
                    enabled: true,
                    interval_hours: 24,
                },
                decay: DecayConfig { enabled: true },
                governance: GovernanceConfig {
                    max_memory: 350,
                    per_type_limits: PerTypeLimits {
                        identity: None,
                        emotion: Some(50),
                        knowledge: Some(250),
                        event: Some(50),
                    },
                },
                consolidation: ConsolidationConfig {
                    enabled: true,
                    deduplication_window_minutes: 30,
                },
            },
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .context(format!("Failed to read config from {:?}", path.as_ref()))?;
        serde_yaml::from_str(&content)
            .context("Failed to parse config YAML")
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .context("Failed to serialize config")?;
        std::fs::write(path.as_ref(), content)
            .context(format!("Failed to write config to {:?}", path.as_ref()))
    }
}
