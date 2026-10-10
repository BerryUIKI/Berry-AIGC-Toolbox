use serde::{Deserialize, Serialize};

/// Frequency and statistics for a prompt keyword.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptStat {
    /// The prompt token or phrase.
    pub text: String,
    /// Number of images using this token.
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptKeywordStat {
    pub keyword: String,
    pub count: usize,
}

/// One snapshot of files contributing usable generation information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PromptInsights {
    pub total_analyzed: usize,
    pub top_positive_words: Vec<PromptKeywordStat>,
    pub top_negative_words: Vec<PromptKeywordStat>,
    pub top_models: Vec<PromptKeywordStat>,
    pub top_samplers: Vec<PromptKeywordStat>,
}
