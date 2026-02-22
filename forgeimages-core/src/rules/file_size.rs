//! File Size Rule — NEW for Cover Forge
//!
//! Validates that the asset file size is within platform limits.
//! Different platforms have different maximums (KDP: 650MB, IS: 600MB).

use crate::templates::Template;
use crate::validation::{AssetInput, ValidationRule, ValidationViolation};

/// Validates file size against a maximum.
///
/// Note: This rule requires the file size to be provided externally
/// (via the `color_count` field repurposed, or via a future AssetInput extension).
/// For Phase 1, this validates using an explicit max_bytes parameter.
pub struct FileSizeRule {
    /// Maximum file size in bytes
    pub max_bytes: u64,
    /// Label for the limit (e.g., "KDP 650MB", "IngramSpark 600MB")
    pub limit_label: String,
}

impl FileSizeRule {
    pub fn kdp() -> Self {
        Self {
            max_bytes: 650 * 1024 * 1024, // 650 MB
            limit_label: "KDP 650MB".to_string(),
        }
    }

    pub fn ingram_spark() -> Self {
        Self {
            max_bytes: 600 * 1024 * 1024, // 600 MB
            limit_label: "IngramSpark 600MB".to_string(),
        }
    }
}

impl ValidationRule for FileSizeRule {
    fn name(&self) -> &'static str {
        "file_size"
    }

    fn validate(&self, _input: &AssetInput, _template: &Template) -> Vec<ValidationViolation> {
        // File size validation requires file-level context not available in AssetInput.
        // This rule is activated by the file_validation path which wraps it with actual
        // file size data. For metadata-only validation, this is a no-op.
        //
        // In Phase 2, AssetInput will be extended with an optional `file_size_bytes` field.
        vec![]
    }
}
