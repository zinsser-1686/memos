//! Relations sidecar — SQLite for causal/semantic/temporal edges and entities

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::Path;
use tracing::{debug, instrument};

/// Relation types between memories
#[derive(Debug, Clone)]
pub enum RelationType {
    Cause,
    Effect,
    Semantic,
    Temporal,
    Contradicts,
    Supports,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::Cause => "cause",
            RelationType::Effect => "effect",
            RelationType::Semantic => "semantic",
            RelationType::Temporal => "temporal",
            RelationType::Contradicts => "contradicts",
            RelationType::Supports => "supports",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "cause" => RelationType::Cause,
            "effect" => RelationType::Effect,
            "semantic" => RelationType::Semantic,
            "temporal" => RelationType::Temporal,
            "contradicts" => RelationType::Contradicts,
            "supports" => RelationType::Supports,
            _ => RelationType::Semantic,
        }
    }
}

/// Relations database sidecar
pub struct RelationsDb {
    conn: Connection,
}

impl RelationsDb {
    /// Create a new relations database
    #[instrument(skip(path))]
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path.as_ref())
            .context("Failed to open relations database")?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS relations (
                id           INTEGER PRIMARY KEY,
                source_id    TEXT NOT NULL,
                target_id    TEXT NOT NULL,
                rel_type     TEXT NOT NULL,
                confidence   REAL DEFAULT 0.0,
                created_at   TEXT,
                UNIQUE(source_id, target_id, rel_type)
            );

            CREATE INDEX IF NOT EXISTS idx_relations_source ON relations(source_id);
            CREATE INDEX IF NOT EXISTS idx_relations_target ON relations(target_id);
            CREATE INDEX IF NOT EXISTS idx_relations_type  ON relations(rel_type);

            CREATE TABLE IF NOT EXISTS entities (
                id           INTEGER PRIMARY KEY,
                name         TEXT NOT NULL UNIQUE,
                entity_type  TEXT,
                canonical    TEXT,
                created_at   TEXT,
                updated_at   TEXT
            );

            CREATE TABLE IF NOT EXISTS entity_mentions (
                id          INTEGER PRIMARY KEY,
                entity_id   INTEGER NOT NULL,
                frame_id    TEXT NOT NULL,
                context     TEXT,
                created_at  TEXT,
                FOREIGN KEY (entity_id) REFERENCES entities(id)
            );
            "#,
        )
        .context("Failed to create relations schema")?;

        debug!("Relations database created");
        Ok(Self { conn })
    }

    /// Open an existing relations database
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path.as_ref())
            .context("Failed to open relations database")?;
        Ok(Self { conn })
    }

    /// Add a relation between two memory frames
    #[instrument(skip(self, source_id, target_id))]
    pub fn add_relation(
        &self,
        source_id: &str,
        target_id: &str,
        rel_type: &str,
        confidence: f32,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO relations (source_id, target_id, rel_type, confidence, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![source_id, target_id, rel_type, confidence, now],
        )
        .context("Failed to insert relation")?;

        debug!("Added relation: {} --{}--> {} (conf: {:.2})", source_id, rel_type, target_id, confidence);
        Ok(())
    }

    /// Get neighbors of a memory frame
    pub fn get_neighbors(
        &self,
        frame_id: &str,
        rel_type: Option<&str>,
    ) -> Result<Vec<(String, f32)>> {
        let query = match rel_type {
            Some(rt) => {
                r#"
                SELECT source_id, confidence FROM relations
                WHERE target_id = ?1 AND rel_type = ?2
                UNION
                SELECT target_id, confidence FROM relations
                WHERE source_id = ?1 AND rel_type = ?2
                "#
            }
            None => {
                r#"
                SELECT source_id, confidence FROM relations WHERE target_id = ?1
                UNION
                SELECT target_id, confidence FROM relations WHERE source_id = ?1
                "#
            }
        };

        let mut stmt = self.conn.prepare(query)
            .context("Failed to prepare neighbors query")?;

        let rows = if let Some(rt) = rel_type {
            stmt.query_map(params![frame_id, rt], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f32>(1)?))
            })
        } else {
            stmt.query_map(params![frame_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f32>(1)?))
            })
        }
        .context("Failed to query neighbors")?;

        let mut neighbors = Vec::new();
        for row in rows {
            neighbors.push(row.context("Failed to read neighbor")?);
        }

        Ok(neighbors)
    }

    /// Add or update an entity
    pub fn upsert_entity(
        &self,
        name: &str,
        entity_type: Option<&str>,
        canonical: Option<&str>,
    ) -> Result<i64> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            r#"
            INSERT INTO entities (name, entity_type, canonical, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?4)
            ON CONFLICT(name) DO UPDATE SET
                entity_type = COALESCE(?2, entity_type),
                canonical = COALESCE(?3, canonical),
                updated_at = ?4
            "#,
            params![name, entity_type, canonical, now],
        )
        .context("Failed to upsert entity")?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Record an entity mention in a memory
    pub fn add_entity_mention(
        &self,
        entity_id: i64,
        frame_id: &str,
        context: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            r#"
            INSERT INTO entity_mentions (entity_id, frame_id, context, created_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![entity_id, frame_id, context, now],
        )
        .context("Failed to insert entity mention")?;

        Ok(())
    }

    /// Get entity by name
    pub fn get_entity(&self, name: &str) -> Result<Option<(i64, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, canonical FROM entities WHERE name = ?1"
        )
        .context("Failed to prepare entity query")?;

        let result = stmt.query_row(params![name], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        });

        match result {
            Ok(entity) => Ok(Some(entity)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_relations_basic() {
        let tmp = NamedTempFile::new().unwrap();
        let db = RelationsDb::create(tmp.path()).unwrap();

        db.add_relation("a", "b", "cause", 0.85).unwrap();
        db.add_relation("a", "c", "contradicts", 0.70).unwrap();

        let neighbors = db.get_neighbors("a", None).unwrap();
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_entity_upsert() {
        let tmp = NamedTempFile::new().unwrap();
        let db = RelationsDb::create(tmp.path()).unwrap();

        let id1 = db.upsert_entity("GPT-4", Some("model"), Some("GPT-4")).unwrap();
        let id2 = db.upsert_entity("GPT-4", None, None).unwrap();

        assert_eq!(id1, id2); // Same entity
    }
}
