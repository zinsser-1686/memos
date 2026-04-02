//! Vault — the main storage interface backed by Memvid's .mv2 file format

use crate::config::Config;
use crate::storage::frame::{MemoryFrame, MemoryType, PutOptions, LifecycleState};
use crate::cognitive::SearchHit;
use anyhow::{Context, Result};
use memvid_core::{Memvid, PutOptions as MemvidPutOptions, SearchRequest as MemvidSearchRequest};
use std::path::Path;
use tracing::{debug, info, instrument};

/// Vault statistics
#[derive(Debug, Default)]
pub struct VaultStats {
    pub total_memories: usize,
    pub active_count: usize,
    pub archived_count: usize,
    pub by_type: std::collections::HashMap<String, usize>,
}

/// The main vault — backed by a single .mv2 file
pub struct Vault {
    mv2: Memvid,
    relations_db: crate::storage::relations::RelationsDb,
    config: Config,
}

impl Vault {
    /// Create a new vault
    #[instrument(skip(path))]
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        info!("Creating vault at {:?}", path);

        let mv2 = Memvid::create(path)
            .context("Failed to create Memvid vault")?;

        // Create relations sidecar
        let db_path = path.with_extension("db");
        let relations_db = crate::storage::relations::RelationsDb::create(&db_path)?;

        let config = Config::default();
        Ok(Self { mv2, relations_db, config })
    }

    /// Open an existing vault
    #[instrument(skip(path))]
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        debug!("Opening vault at {:?}", path);

        let mv2 = Memvid::open(path)
            .context("Failed to open Memvid vault")?;

        let db_path = path.with_extension("db");
        let relations_db = if db_path.exists() {
            crate::storage::relations::RelationsDb::open(&db_path)?
        } else {
            crate::storage::relations::RelationsDb::create(&db_path)?
        };

        let config = Config::default();
        Ok(Self { mv2, relations_db, config })
    }

    /// Put a new memory into the vault
    #[instrument(skip(self, content, opts))]
    pub fn put(&mut self, content: &[u8], opts: PutOptions) -> Result<String> {
        let content_str = String::from_utf8_lossy(content);
        let mut frame = MemoryFrame::new(content_str.as_ref());

        // Apply options
        if let Some(title) = opts.title {
            frame.title = title;
        }
        if let Some(mtype) = opts.memory_type {
            frame.memory_type = mtype;
            frame.confidence = mtype.baseline_confidence();
        }
        if let Some(tags) = opts.tags {
            frame.tags = tags;
        }
        if let Some(uri) = opts.uri {
            frame.uri = uri;
        }
        if let Some(session) = opts.source_session {
            frame.source_session = Some(session);
        }
        if let Some(replaces) = opts.replaces {
            frame.replaces = replaces;
        }
        if let Some(observed) = opts.observed_at {
            frame.observed_at = Some(observed);
        }

        // Serialize frame
        let json = serde_json::to_string(&frame)
            .context("Failed to serialize memory frame")?;

        // Put into Memvid
        let mut mv_opts = MemvidPutOptions::builder();
        if !frame.title.is_empty() {
            mv_opts.title(&frame.title);
        }
        mv_opts.tag("type", frame.memory_type.as_str());
        for tag in &frame.tags {
            mv_opts.tag("tag", tag);
        }

        self.mv2.put_bytes_with_options(json.as_bytes(), &mv_opts.build())?;
        self.mv2.commit()?;

        info!("Stored memory frame {}", frame.frame_id);
        Ok(frame.frame_id)
    }

    /// Search the vault
    #[instrument(skip(self, request))]
    pub fn search(&self, request: crate::retrieval::SearchRequest) -> Result<Vec<SearchHit>> {
        debug!("Searching vault with query: {}", request.query);

        let mv_request = MemvidSearchRequest {
            query: request.query.clone(),
            top_k: request.top_k,
            snippet_chars: 200,
            ..Default::default()
        };

        let response = self.mv2.search(mv_request)?;

        let hits: Vec<SearchHit> = response.hits
            .into_iter()
            .map(|hit| {
                // Parse the frame from the stored JSON
                let frame: MemoryFrame = serde_json::from_slice(&hit.bytes)
                    .unwrap_or_else(|_| MemoryFrame::new(""));

                SearchHit {
                    frame_id: frame.frame_id.clone(),
                    title: frame.title.clone(),
                    snippet: hit.text,
                    memory_type: frame.memory_type.as_str().to_string(),
                    score: hit.score,
                    confidence: frame.confidence,
                    tags: frame.tags,
                    created_at: frame.created_at,
                }
            })
            .collect();

        Ok(hits)
    }

    /// Get a single memory by ID
    pub fn get(&self, frame_id: &str) -> Result<Option<MemoryFrame>> {
        // Search for the frame
        let request = MemvidSearchRequest {
            query: format!("frame_id:{}", frame_id),
            top_k: 1,
            ..Default::default()
        };

        let response = self.mv2.search(request)?;

        if response.hits.is_empty() {
            return Ok(None);
        }

        let frame: MemoryFrame = serde_json::from_slice(&response.hits[0].bytes)
            .context("Failed to parse memory frame")?;

        Ok(Some(frame))
    }

    /// Get vault statistics
    pub fn stats(&self) -> Result<VaultStats> {
        // This would scan the vault to count memories by type
        // For now, return placeholder
        Ok(VaultStats::default())
    }

    /// Add a causal relation between two memories
    pub fn add_relation(&self, source: &str, target: &str, rel_type: &str, confidence: f32) -> Result<()> {
        self.relations_db.add_relation(source, target, rel_type, confidence)
    }

    /// Get causal neighbors of a memory
    pub fn get_causal_neighbors(&self, frame_id: &str, rel_type: Option<&str>) -> Result<Vec<(String, f32)>> {
        self.relations_db.get_neighbors(frame_id, rel_type)
    }
}
