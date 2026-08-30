use serde::{Deserialize, Serialize};

use crate::{FileSortField, SortDirection};

/// Criteria for filtering and querying files in the library.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchCriteria {
    /// Broad text query matching across prompt, negative_prompt, model_name, and path.
    pub text: Option<String>,
    /// Filter specifically by prompt content (partial match).
    pub prompt: Option<String>,
    /// Filter specifically by negative prompt content (partial match).
    pub negative_prompt: Option<String>,
    /// Filter by model name (partial match).
    pub model_name: Option<String>,
    /// Filter by model hash (partial match).
    pub model_hash: Option<String>,
    /// Filter by sampler name (partial match).
    pub sampler: Option<String>,
    /// Minimum generation steps.
    pub min_steps: Option<u32>,
    /// Maximum generation steps.
    pub max_steps: Option<u32>,
    /// Minimum CFG scale.
    pub min_cfg: Option<f64>,
    /// Maximum CFG scale.
    pub max_cfg: Option<f64>,
    /// Minimum user rating (1–10).
    pub min_rating: Option<u8>,
    /// Maximum user rating (1–10).
    pub max_rating: Option<u8>,
    /// Minimum aesthetic score.
    pub min_aesthetic: Option<f64>,
    /// Maximum aesthetic score.
    pub max_aesthetic: Option<f64>,
    /// Optional folder constraint. If `None`, searches across all indexed folders.
    pub folder_id: Option<i64>,
    /// Field to sort results by. Defaults to `ModifiedAt`.
    pub sort: Option<FileSortField>,
    /// Sort direction. Defaults to `Desc`.
    pub direction: Option<SortDirection>,
    /// Maximum number of records to return.
    pub limit: Option<usize>,
    /// Number of records to skip (for pagination).
    pub offset: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_criteria_serde_roundtrip() {
        let criteria = SearchCriteria {
            text: Some("cyberpunk".to_string()),
            prompt: Some("neon cityscape".to_string()),
            negative_prompt: Some("blurry".to_string()),
            model_name: Some("dreamshaper".to_string()),
            model_hash: Some("abc12345".to_string()),
            sampler: Some("Euler a".to_string()),
            min_steps: Some(20),
            max_steps: Some(50),
            min_cfg: Some(7.0),
            max_cfg: Some(12.5),
            min_rating: Some(8),
            max_rating: Some(10),
            min_aesthetic: Some(0.6),
            max_aesthetic: Some(0.95),
            folder_id: Some(42),
            sort: Some(FileSortField::Rating),
            direction: Some(SortDirection::Desc),
            limit: Some(100),
            offset: Some(0),
        };

        let json = serde_json::to_string(&criteria).unwrap();
        let decoded: SearchCriteria = serde_json::from_str(&json).unwrap();
        assert_eq!(criteria, decoded);
    }

    #[test]
    fn empty_criteria_defaults() {
        let criteria = SearchCriteria::default();
        assert!(criteria.text.is_none());
        assert!(criteria.prompt.is_none());
        assert!(criteria.min_rating.is_none());
        assert!(criteria.sort.is_none());
    }
}
