use serde::{Deserialize, Serialize};

/// Target image container format for image transformation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransformFormat {
    #[default]
    Original,
    Jpeg,
    Webp,
    Png,
    Avif,
}

/// Metadata handling policy for image transformations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransformMetadataPolicy {
    #[default]
    KeepSupported,
    StripAi,
    StripAll,
}

/// Collision policy when staged output filename already exists in the target directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransformCollisionPolicy {
    #[default]
    Rename,
    Skip,
}

/// Specification of image transformation parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformSpec {
    pub format: TransformFormat,
    #[serde(default)]
    pub quality: Option<u8>,
    #[serde(default)]
    pub max_edge: Option<u32>,
    #[serde(default)]
    pub scale_percent: Option<u32>,
    #[serde(default)]
    pub align_multiple: Option<u32>,
    #[serde(default)]
    pub target_size_kb: Option<u32>,
    #[serde(default)]
    pub metadata_policy: TransformMetadataPolicy,
    #[serde(default)]
    pub collision_policy: TransformCollisionPolicy,
}

impl Default for TransformSpec {
    fn default() -> Self {
        Self {
            format: TransformFormat::Original,
            quality: None,
            max_edge: None,
            scale_percent: None,
            align_multiple: None,
            target_size_kb: None,
            metadata_policy: TransformMetadataPolicy::KeepSupported,
            collision_policy: TransformCollisionPolicy::Rename,
        }
    }
}

impl TransformSpec {
    /// Only an unmodified Original/KeepSupported request can copy source bytes.
    /// Collision policy affects publication, not whether pixels/metadata need processing.
    pub fn requires_processing(&self) -> bool {
        self.format != TransformFormat::Original
            || self.quality.is_some()
            || self.max_edge.is_some()
            || self.scale_percent.is_some()
            || self.align_multiple.is_some()
            || self.target_size_kb.is_some()
            || self.metadata_policy != TransformMetadataPolicy::KeepSupported
    }
}

/// Source disposition for managed import transformation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImportSourceDisposition {
    #[default]
    Keep,
}

/// Request for importing external images into a managed vault with optional transform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportTransformRequest {
    pub managed_destination_id: i64,
    pub source_paths: Vec<String>,
    pub spec: TransformSpec,
    #[serde(default)]
    pub source_disposition: ImportSourceDisposition,
}

/// Original disposition for library batch transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OriginalDisposition {
    #[default]
    Keep,
    Archive,
    Trash,
}

/// Request for batch compressing / converting existing managed library assets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryTransformRequest {
    pub file_ids: Vec<i64>,
    pub spec: TransformSpec,
    #[serde(default)]
    pub original_disposition: OriginalDisposition,
}

/// Status of an individual item in a transformation job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformItemStatus {
    Succeeded,
    Failed,
    Skipped,
    Canceled,
}

/// Receipt record for an individual transformed item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformItemReceipt {
    pub source_id_or_path: String,
    pub output_id_or_path: Option<String>,
    pub status: TransformItemStatus,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub original_action: Option<String>,
}

/// Full receipt for an executed transformation job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformJobReceipt {
    pub job_id: String,
    pub phase: String,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub canceled: usize,
    pub items: Vec<TransformItemReceipt>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_spec_serde_roundtrip() {
        let spec = TransformSpec {
            format: TransformFormat::Jpeg,
            quality: Some(85),
            max_edge: Some(2048),
            scale_percent: Some(75),
            align_multiple: Some(8),
            target_size_kb: Some(500),
            metadata_policy: TransformMetadataPolicy::StripAi,
            collision_policy: TransformCollisionPolicy::Rename,
        };
        let json = serde_json::to_string(&spec).unwrap();
        let parsed: TransformSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, parsed);

        // Backward compatibility: JSON missing scale_percent, align_multiple, target_size_kb
        let legacy_json = r#"{
            "format": "webp",
            "quality": 80,
            "max_edge": 1920,
            "metadata_policy": "keep_supported",
            "collision_policy": "skip"
        }"#;
        let legacy_parsed: TransformSpec = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(legacy_parsed.format, TransformFormat::Webp);
        assert_eq!(legacy_parsed.scale_percent, None);
        assert_eq!(legacy_parsed.align_multiple, None);
        assert_eq!(legacy_parsed.target_size_kb, None);
    }

    #[test]
    fn transform_job_receipt_serde_roundtrip() {
        let receipt = TransformJobReceipt {
            job_id: "job-123".to_string(),
            phase: "completed".to_string(),
            total: 1,
            succeeded: 1,
            failed: 0,
            skipped: 0,
            canceled: 0,
            items: vec![TransformItemReceipt {
                source_id_or_path: "/path/to/img.png".to_string(),
                output_id_or_path: Some("/vault/img.webp".to_string()),
                status: TransformItemStatus::Succeeded,
                error_code: None,
                original_action: Some("kept".to_string()),
            }],
        };
        let json = serde_json::to_string(&receipt).unwrap();
        let parsed: TransformJobReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(receipt, parsed);
    }
}
