//! Memos CLI — Entry point

use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod storage;
mod retrieval;
mod cognitive;
mod lifecycle;
mod hooks;
mod api;

use memos::{Vault, Config, SearchRequest, QueryIntent};

#[derive(Parser)]
#[command(
    name = "memos",
    version = "0.1.0",
    about = "AI Agent Memory System: high-performance storage + cognitive intelligence"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to config file
    #[arg(short, long, default_value = "memos.yaml")]
    config: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new vault
    Init {
        /// Path for the vault file
        #[arg(default_value = "memos.mv2")]
        path: String,
    },

    /// Add a memory to the vault
    Put {
        /// Memory content
        content: String,

        /// Title for this memory
        #[arg(short, long)]
        title: Option<String>,

        /// Memory type (decision, note, handoff, etc.)
        #[arg(short, long, default_value = "note")]
        memory_type: String,

        /// Comma-separated tags
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// Search memories (BM25 + vector hybrid)
    Search {
        /// Search query
        query: String,

        /// Maximum results
        #[arg(short, long, default_value_t = 10)]
        top_k: usize,

        /// Intent hint (WHY, WHEN, ENTITY, WHAT)
        #[arg(short, long)]
        intent: Option<String>,
    },

    /// Full retrieval pipeline (search + rerank + graph expansion)
    Query {
        query: String,
        #[arg(short, long, default_value_t = 10)]
        top_k: usize,
    },

    /// Intent-classified causal search
    Intent {
        query: String,
        #[arg(short, long, default_value_t = 10)]
        top_k: usize,
    },

    /// Show temporal neighborhood around a memory
    Timeline {
        /// Document ID
        doc_id: String,

        /// Number of memories before
        #[arg(long, default_value_t = 3)]
        before: usize,

        /// Number of memories after
        #[arg(long, default_value_t = 3)]
        after: usize,
    },

    /// Trace causal chain from a memory
    Causal {
        /// Starting document ID
        doc_id: String,

        /// Maximum hops
        #[arg(long, default_value_t = 3)]
        hops: usize,
    },

    /// Extract decisions from a transcript
    Extract {
        /// Session transcript
        transcript: String,

        /// Session ID (optional)
        #[arg(long)]
        session_id: Option<String>,
    },

    /// Run A-MEM consolidation + feedback processing
    Reflect {
        /// Dry run (no changes)
        #[arg(long)]
        dry_run: bool,
    },

    /// Record feedback for surfaced memories
    Feedback {
        /// Document ID
        doc_id: String,

        /// Feedback type
        #[arg(value_enum)]
        feedback_type: FeedbackType,
    },

    /// Soft decay or hard delete a memory
    Forget {
        /// Search query to find memory
        query: String,

        /// Hard delete (default: soft decay)
        #[arg(long)]
        hard: bool,
    },

    /// Pin a memory for priority boosting
    Pin {
        /// Document ID
        doc_id: String,

        /// Priority (0.0 - 1.0)
        #[arg(long, default_value_t = 0.3)]
        priority: f32,
    },

    /// Snooze a memory temporarily
    Snooze {
        /// Document ID
        doc_id: String,

        /// Snooze until date (YYYY-MM-DD)
        until: String,
    },

    /// Show vault statistics
    Status,

    /// Health check
    Doctor,
}

#[derive(clap::ValueEnum, Clone)]
enum FeedbackType {
    Helpful,
    Neutral,
    Harmful,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"))
    } else {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = if std::path::Path::new(&cli.config).exists() {
        Config::load(&cli.config)?
    } else {
        Config::default()
    };

    match cli.command {
        Commands::Init { path } => {
            tracing::info!("Initializing vault at {}", path);
            Vault::create(&path)?;
            tracing::info!("Vault created successfully");
        }

        Commands::Put { content, title, memory_type, tags } => {
            let mut vault = Vault::open(&config.vault_path)?;
            let opts = storage::PutOptions::default()
                .title(title.as_deref())
                .memory_type(&memory_type)
                .tags(tags.map(|t| t.split(',').map(String::from).collect()));

            vault.put(content.as_bytes(), opts)?;
            tracing::info!("Memory stored successfully");
        }

        Commands::Search { query, top_k, intent } => {
            let vault = Vault::open(&config.vault_path)?;
            let req = SearchRequest {
                query,
                top_k,
                intent: intent.map(|i| match i.to_uppercase().as_str() {
                    "WHY" => QueryIntent::Why,
                    "WHEN" => QueryIntent::When,
                    "ENTITY" => QueryIntent::Entity,
                    _ => QueryIntent::What,
                }),
                ..Default::default()
            };

            let results = vault.search(req)?;
            for (i, hit) in results.iter().enumerate() {
                println!("{}. [{}] {} (score: {:.3})", i + 1, hit.memory_type, hit.title, hit.score);
            }
        }

        Commands::Status => {
            let vault = Vault::open(&config.vault_path)?;
            let stats = vault.stats()?;
            println!("Vault: {}", config.vault_path);
            println!("Total memories: {}", stats.total_memories);
            println!("Active: {}", stats.active_count);
            println!("Archived: {}", stats.archived_count);
            println!("By type:");
            for (mtype, count) in &stats.by_type {
                println!("  {}: {}", mtype, count);
            }
        }

        Commands::Doctor => {
            println!("Memos v{}", env!("CARGO_PKG_VERSION"));
            println!("Checking system...");
            println!("✓ Vault: {}", config.vault_path);
            println!("✓ Config loaded");
            // TODO: Check embedding/LLM server connectivity
        }

        _ => {
            tracing::warn!("Command not yet implemented: {:?}", cli.command);
            println!("This command is planned but not yet implemented.");
        }
    }

    Ok(())
}
