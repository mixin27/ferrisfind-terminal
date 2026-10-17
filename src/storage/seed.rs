use uuid::Uuid;

use super::StorageError;
use crate::domain::{
    fictional_records,
    persistence::{RecordDraft, SourceDraft, SourceKind},
};

pub const DEFAULT_SEED_COUNT: u32 = 1_000;
pub const MAX_SEED_COUNT: u32 = 100_000;

pub fn source() -> SourceDraft {
    SourceDraft {
        id: Uuid::new_v5(&Uuid::NAMESPACE_OID, b"ferrisfind.fictional.seed.v1"),
        name: "Fictional workshop seed v1".into(),
        kind: SourceKind::Seed,
    }
}

/// Deterministic IDs and content without RNG state; retains only 24 templates.
pub fn records(count: u32) -> Result<impl Iterator<Item = RecordDraft>, StorageError> {
    if !(1..=MAX_SEED_COUNT).contains(&count) {
        return Err(StorageError::InvalidSeedCount);
    }
    let templates = fictional_records();
    let source_id = source().id;
    Ok((0..count).map(move |index| {
        let template = &templates[index as usize % templates.len()];
        RecordDraft {
            id: Uuid::new_v5(&source_id, &index.to_be_bytes()),
            source_id,
            title: format!("Fictional {:06} — {}", index + 1, template.title),
            category: template.category.into(),
            content: format!("{}\nFictional seed v1 sample{:06}.", template.content, index + 1),
            metadata: serde_json::json!({"fictional": true, "generator": "v1", "ordinal": index + 1}),
        }
    }))
}
