use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    Seed,
    Json,
    Csv,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Seed => "seed",
            Self::Json => "json",
            Self::Csv => "csv",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceDraft {
    pub id: Uuid,
    pub name: String,
    pub kind: SourceKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub draft: SourceDraft,
    pub created_at: String,
    pub record_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecordDraft {
    pub id: Uuid,
    pub source_id: Uuid,
    pub title: String,
    pub category: String,
    pub content: String,
    pub metadata: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredRecord {
    pub draft: RecordDraft,
    pub created_at: String,
}
