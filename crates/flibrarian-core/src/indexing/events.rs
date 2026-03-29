use serde::Serialize;

use super::IndexingPhase;

#[derive(Clone, Serialize)]
pub struct IndexingProgress {
    pub phase: IndexingPhase,
    pub current: usize,
    pub total: usize,
}

#[derive(Clone, Serialize)]
pub struct IndexingWarning {
    pub message: String,
}

#[derive(Clone, Serialize)]
pub struct IndexingInfo {
    pub message: String,
}
