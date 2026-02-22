//! Resolution Rule — extracted from validation.rs
//!
//! Checks minimum width and height requirements.

use crate::templates::Template;
use crate::validation::{AssetInput, ValidationRule, ValidationViolation, ViolationSeverity};

pub struct ResolutionRule;

impl ValidationRule for ResolutionRule {
    fn name(&self) -> &'static str {
        "resolution"
    }

    fn validate(&self, input: &AssetInput, template: &Template) -> Vec<ValidationViolation> {
        if !template.validation.rules.resolution.enabled {
            return vec![];
        }

        let min_w = template.validation.rules.resolution.min_width;
        let min_h = template.validation.rules.resolution.min_height;

        if input.width < min_w || input.height < min_h {
            vec![ValidationViolation {
                rule: self.name().to_string(),
                severity: ViolationSeverity::Error,
                message: "Resolution too low".to_string(),
                expected: Some(format!("{}x{} minimum", min_w, min_h)),
                actual: Some(format!("{}x{}", input.width, input.height)),
                remediation: vec!["Provide higher resolution source image".to_string()],
            }]
        } else {
            vec![]
        }
    }
}
