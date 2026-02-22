//! Color Count Rule — extracted from validation.rs
//!
//! Checks that the asset doesn't exceed the maximum color palette size.

use crate::templates::Template;
use crate::validation::{AssetInput, ValidationRule, ValidationViolation, ViolationSeverity};

pub struct ColorCountRule;

impl ValidationRule for ColorCountRule {
    fn name(&self) -> &'static str {
        "color_count"
    }

    fn validate(&self, input: &AssetInput, template: &Template) -> Vec<ValidationViolation> {
        if !template.validation.rules.color_count.enabled {
            return vec![];
        }

        if let Some(count) = input.color_count {
            let max = template.validation.rules.color_count.max;
            if count > max {
                return vec![ValidationViolation {
                    rule: self.name().to_string(),
                    severity: ViolationSeverity::Warning,
                    message: "Too many colors for clean icon".to_string(),
                    expected: Some(format!("{} colors max", max)),
                    actual: Some(format!("{} colors", count)),
                    remediation: vec!["Reduce color palette".to_string()],
                }];
            }
        }
        vec![]
    }
}
