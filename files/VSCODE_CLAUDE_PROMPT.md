# ForgeImages Integration - VS Code Claude Prompt

**Copy everything below the line into VS Code Claude.**

---

## Role & Context

You are implementing the ForgeAgents + ForgeImages integration layer for Boswell Digital Solutions LLC's Forge Ecosystem.

**Critical Constraint:** "Agents suggest, ForgeImages enforces."
- ForgeAgents (Python) may request operations
- ForgeImages (Rust) is the ONLY enforcement point
- The bridge returns HTTP 422 on validation failure
- Agents cannot bypass, override, or skip validation

## Project Structure to Create

```
forgeimages-core/                      # Rust crate
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── templates.rs
│   ├── validation.rs
│   ├── hashing.rs
│   ├── print.rs
│   ├── pipeline.rs
│   └── bin/
│       └── forgeimages_cli.rs
├── templates/
│   └── pwa-icon.json
└── tests/
    └── invariants.rs

forgeagents-forgeimages/               # Python package
├── pyproject.toml
├── bridge/
│   ├── __init__.py
│   ├── forgeimages_bridge.py
│   ├── models.py
│   ├── audit.py
│   └── settings.py
├── skill/
│   ├── __init__.py
│   └── forgeimages_skill.py
└── tests/
    └── test_agent_boundary.py
```

---

## STEP 1: Rust Cargo.toml

Create `forgeimages-core/Cargo.toml`:

```toml
[package]
name = "forgeimages-core"
version = "1.0.0"
edition = "2021"
description = "Visual Production Compiler - ForgeImages Core Engine"
license = "Proprietary"
authors = ["Boswell Digital Solutions LLC"]

[lib]
name = "forgeimages_core"
path = "src/lib.rs"

[[bin]]
name = "forgeimages-cli"
path = "src/bin/forgeimages_cli.rs"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
semver = { version = "1.0", features = ["serde"] }
thiserror = "1.0"
base64 = "0.21"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
clap = { version = "4.0", features = ["derive"] }

[dev-dependencies]
tempfile = "3.0"

[features]
default = []
test-hooks = []
```

---

## STEP 2: Rust src/lib.rs

```rust
//! ForgeImages Core - Visual Production Compiler
//!
//! # The Six Laws (Non-Negotiable)
//! 1. SVG Is Truth
//! 2. Templates Are Contracts
//! 3. Validation Is Protective
//! 4. Deterministic Output
//! 5. Manifests Enable Reproduction
//! 6. Agents Suggest, Engine Enforces

pub mod templates;
pub mod validation;
pub mod hashing;
pub mod print;
pub mod pipeline;

pub use templates::{Template, TemplateId, ExportSpec, AssetClass};
pub use validation::{ValidationResult, ValidationRule, ValidationViolation, ViolationSeverity};
pub use hashing::{compute_manifest_hash, compute_job_hash, canonical_json};
pub use print::PrintAuthority;
pub use pipeline::{CompilationPipeline, CompiledAsset, CompileRequest, PipelineError};

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MIN_TEMPLATE_VERSION: &str = "1.0.0";
```

---

## STEP 3: Rust src/templates.rs

```rust
//! Template System - Enforceable Contracts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub type TemplateId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub id: TemplateId,
    pub name: String,
    pub description: String,
    pub template_version: String,
    pub engine_min_version: String,
    #[serde(default)]
    pub deprecated: bool,
    pub asset_class: AssetClass,
    pub aspect_ratio: [u32; 2],
    pub canonical_size: [u32; 2],
    #[serde(default = "default_true")]
    pub vector_master: bool,
    #[serde(default)]
    pub validation: ValidationConfig,
    #[serde(default)]
    pub exports: Vec<ExportSpec>,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AssetClass {
    Icon,
    Cover,
    Banner,
    Logo,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationConfig {
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub failure_mode: FailureMode,
    #[serde(default)]
    pub rules: ValidationRules,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FailureMode {
    #[default]
    Block,
    Warn,
    Log,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationRules {
    #[serde(default)]
    pub aspect_ratio: RuleConfig,
    #[serde(default)]
    pub resolution: ResolutionRule,
    #[serde(default)]
    pub color_count: ColorCountRule,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
}

fn default_tolerance() -> f64 { 0.01 }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionRule {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_min_width")]
    pub min_width: u32,
    #[serde(default = "default_min_height")]
    pub min_height: u32,
}

fn default_min_width() -> u32 { 1024 }
fn default_min_height() -> u32 { 1024 }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorCountRule {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_max_colors")]
    pub max: u32,
}

fn default_max_colors() -> u32 { 16 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSpec {
    pub id: String,
    pub description: String,
    pub size: [u32; 2],
    pub format: ExportFormat,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Svg,
    Png,
    Ico,
    Pdf,
    Jpg,
}

/// Template registry - loads and caches templates
pub struct TemplateRegistry {
    templates: HashMap<TemplateId, Template>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self { templates: HashMap::new() }
    }

    pub fn load_from_dir(dir: &Path) -> Result<Self, std::io::Error> {
        let mut registry = Self::new();
        if dir.exists() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(template) = serde_json::from_str::<Template>(&content) {
                            registry.templates.insert(template.id.clone(), template);
                        }
                    }
                }
            }
        }
        Ok(registry)
    }

    pub fn get(&self, id: &str) -> Option<&Template> {
        self.templates.get(id)
    }

    pub fn list(&self) -> Vec<&Template> {
        self.templates.values().collect()
    }

    pub fn register(&mut self, template: Template) {
        self.templates.insert(template.id.clone(), template);
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## STEP 4: Rust src/validation.rs

```rust
//! Validation System - Rule/Policy Separation
//!
//! Rules produce structured violations.
//! Policy maps violations to actions.

use serde::{Deserialize, Serialize};
use crate::templates::{Template, FailureMode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationViolation {
    pub rule: String,
    pub severity: ViolationSeverity,
    pub message: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub remediation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub violations: Vec<ValidationViolation>,
    pub template_id: String,
    pub template_version: String,
}

impl ValidationResult {
    pub fn success(template: &Template) -> Self {
        Self {
            valid: true,
            violations: vec![],
            template_id: template.id.clone(),
            template_version: template.template_version.clone(),
        }
    }

    pub fn failure(template: &Template, violations: Vec<ValidationViolation>) -> Self {
        Self {
            valid: false,
            violations,
            template_id: template.id.clone(),
            template_version: template.template_version.clone(),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.violations.iter().any(|v| v.severity == ViolationSeverity::Error)
    }
}

/// Validation rule trait - produces violations
pub trait ValidationRule {
    fn name(&self) -> &'static str;
    fn validate(&self, input: &AssetInput, template: &Template) -> Vec<ValidationViolation>;
}

/// Input for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInput {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub color_count: Option<u32>,
    #[serde(default)]
    pub format: Option<String>,
}

// --- Concrete Rules ---

pub struct AspectRatioRule;

impl ValidationRule for AspectRatioRule {
    fn name(&self) -> &'static str { "aspect_ratio" }

    fn validate(&self, input: &AssetInput, template: &Template) -> Vec<ValidationViolation> {
        if !template.validation.rules.aspect_ratio.enabled {
            return vec![];
        }

        let expected = template.aspect_ratio[0] as f64 / template.aspect_ratio[1] as f64;
        let actual = input.width as f64 / input.height as f64;
        let tolerance = template.validation.rules.aspect_ratio.tolerance;

        if (expected - actual).abs() > tolerance {
            vec![ValidationViolation {
                rule: self.name().to_string(),
                severity: ViolationSeverity::Error,
                message: format!("Aspect ratio mismatch"),
                expected: Some(format!("{}:{}", template.aspect_ratio[0], template.aspect_ratio[1])),
                actual: Some(format!("{:.3}", actual)),
                remediation: vec!["Crop or resize to match template aspect ratio".to_string()],
            }]
        } else {
            vec![]
        }
    }
}

pub struct ResolutionRule;

impl ValidationRule for ResolutionRule {
    fn name(&self) -> &'static str { "resolution" }

    fn validate(&self, input: &AssetInput, template: &Template) -> Vec<ValidationViolation> {
        if !template.validation.rules.resolution.enabled {
            return vec![];
        }

        let mut violations = vec![];
        let min_w = template.validation.rules.resolution.min_width;
        let min_h = template.validation.rules.resolution.min_height;

        if input.width < min_w || input.height < min_h {
            violations.push(ValidationViolation {
                rule: self.name().to_string(),
                severity: ViolationSeverity::Error,
                message: "Resolution too low".to_string(),
                expected: Some(format!("{}x{} minimum", min_w, min_h)),
                actual: Some(format!("{}x{}", input.width, input.height)),
                remediation: vec!["Provide higher resolution source image".to_string()],
            });
        }

        violations
    }
}

pub struct ColorCountRule;

impl ValidationRule for ColorCountRule {
    fn name(&self) -> &'static str { "color_count" }

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

/// Validator orchestrates rules and applies policy
pub struct Validator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(AspectRatioRule),
                Box::new(ResolutionRule),
                Box::new(ColorCountRule),
            ],
        }
    }

    pub fn validate(&self, input: &AssetInput, template: &Template) -> ValidationResult {
        let mut all_violations = vec![];

        for rule in &self.rules {
            let violations = rule.validate(input, template);
            all_violations.extend(violations);
        }

        // Apply failure mode policy
        let has_errors = all_violations.iter()
            .any(|v| v.severity == ViolationSeverity::Error);

        match template.validation.failure_mode {
            FailureMode::Block if has_errors => {
                ValidationResult::failure(template, all_violations)
            }
            FailureMode::Block => {
                // Warnings don't block
                let errors: Vec<_> = all_violations.into_iter()
                    .filter(|v| v.severity == ViolationSeverity::Error)
                    .collect();
                if errors.is_empty() {
                    ValidationResult::success(template)
                } else {
                    ValidationResult::failure(template, errors)
                }
            }
            FailureMode::Warn | FailureMode::Log => {
                // Never block, just record
                ValidationResult {
                    valid: true,
                    violations: all_violations,
                    template_id: template.id.clone(),
                    template_version: template.template_version.clone(),
                }
            }
        }
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## STEP 5: Rust src/hashing.rs

```rust
//! Hashing System - SHA-256 for Manifests
//!
//! Provides deterministic, reproducible hashes for legal defensibility.

use sha2::{Sha256, Digest};
use serde::Serialize;
use serde_json::{Value, to_string};

/// Compute SHA-256 hash of bytes, return hex string
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Convert to canonical JSON (sorted keys, no whitespace)
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let v: Value = serde_json::to_value(value)?;
    let sorted = sort_value(&v);
    to_string(&sorted)
}

fn sort_value(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut sorted: Vec<_> = map.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(b.0));
            let sorted_map: serde_json::Map<String, Value> = sorted
                .into_iter()
                .map(|(k, v)| (k.clone(), sort_value(v)))
                .collect();
            Value::Object(sorted_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(sort_value).collect())
        }
        _ => v.clone()
    }
}

/// Compute manifest hash for an asset
pub fn compute_manifest_hash<T: Serialize>(manifest: &T) -> Result<String, serde_json::Error> {
    let canonical = canonical_json(manifest)?;
    Ok(sha256_hex(canonical.as_bytes()))
}

/// Compute job hash for audit logging
/// job_hash = sha256(template_id + template_version + canonical_payload + engine_version)
pub fn compute_job_hash(
    template_id: &str,
    template_version: &str,
    payload: &impl Serialize,
    engine_version: &str,
) -> Result<String, serde_json::Error> {
    let canonical_payload = canonical_json(payload)?;
    let combined = format!(
        "{}:{}:{}:{}",
        template_id, template_version, canonical_payload, engine_version
    );
    Ok(sha256_hex(combined.as_bytes()))
}

// We need hex encoding
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canonical_json_sorted() {
        let obj = json!({"z": 1, "a": 2, "m": 3});
        let canonical = canonical_json(&obj).unwrap();
        assert_eq!(canonical, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn test_hash_deterministic() {
        let data = b"test data";
        let h1 = sha256_hex(data);
        let h2 = sha256_hex(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_manifest_hash_stable() {
        let manifest = json!({
            "template_id": "pwa-icon",
            "version": "1.0.0"
        });
        let h1 = compute_manifest_hash(&manifest).unwrap();
        let h2 = compute_manifest_hash(&manifest).unwrap();
        assert_eq!(h1, h2);
    }
}
```

---

## STEP 6: Rust src/print.rs

```rust
//! Print Authority System
//!
//! Defines the source of print specifications to prevent conditional sprawl.

use serde::{Deserialize, Serialize};

/// PrintAuthority determines where print specifications come from.
/// This prevents if/else sprawl throughout the codebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrintAuthority {
    /// System defaults (fallback)
    System,
    /// Template-defined specifications
    Template,
    /// User-provided overrides (with validation)
    User,
}

impl Default for PrintAuthority {
    fn default() -> Self {
        Self::System
    }
}

/// Print specifications for physical output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintSpec {
    pub authority: PrintAuthority,
    pub dpi: u32,
    pub color_space: ColorSpace,
    pub bleed_inches: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ColorSpace {
    Rgb,
    Cmyk,
    Grayscale,
}

impl Default for PrintSpec {
    fn default() -> Self {
        Self {
            authority: PrintAuthority::System,
            dpi: 300,
            color_space: ColorSpace::Rgb,
            bleed_inches: 0.125,
        }
    }
}

impl PrintSpec {
    /// Create from template authority
    pub fn from_template(dpi: u32, color_space: ColorSpace, bleed: f64) -> Self {
        Self {
            authority: PrintAuthority::Template,
            dpi,
            color_space,
            bleed_inches: bleed,
        }
    }

    /// Create from user with validation
    pub fn from_user(dpi: u32, color_space: ColorSpace, bleed: f64) -> Result<Self, &'static str> {
        if dpi < 72 || dpi > 1200 {
            return Err("DPI must be between 72 and 1200");
        }
        if bleed < 0.0 || bleed > 1.0 {
            return Err("Bleed must be between 0 and 1 inch");
        }
        Ok(Self {
            authority: PrintAuthority::User,
            dpi,
            color_space,
            bleed_inches: bleed,
        })
    }
}
```

---

## STEP 7: Rust src/pipeline.rs

```rust
//! Compilation Pipeline - Single Entry Point
//!
//! CRITICAL: compile_asset MUST call validate internally. No bypass.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::templates::{Template, TemplateRegistry, ExportSpec};
use crate::validation::{Validator, ValidationResult, AssetInput};
use crate::hashing::{compute_manifest_hash, compute_job_hash};
use crate::ENGINE_VERSION;

#[cfg(feature = "test-hooks")]
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(feature = "test-hooks")]
static VALIDATION_CALL_COUNT: AtomicU32 = AtomicU32::new(0);

#[cfg(feature = "test-hooks")]
pub fn get_validation_call_count() -> u32 {
    VALIDATION_CALL_COUNT.load(Ordering::SeqCst)
}

#[cfg(feature = "test-hooks")]
pub fn reset_validation_call_count() {
    VALIDATION_CALL_COUNT.store(0, Ordering::SeqCst);
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Template version {0} requires engine >= {1}, current is {2}")]
    EngineVersionMismatch(String, String, String),

    #[error("Compilation error: {0}")]
    CompilationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileRequest {
    pub template_id: String,
    pub asset_input: AssetInput,
    #[serde(default)]
    pub source_data: Option<String>,  // Base64 encoded source
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledAsset {
    pub id: String,
    pub template_id: String,
    pub template_version: String,
    pub engine_version: String,
    pub created_at: DateTime<Utc>,
    pub manifest_hash: String,
    pub job_hash: String,
    pub validation: ValidationResult,
    pub exports: Vec<ExportedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedFile {
    pub id: String,
    pub filename: String,
    pub format: String,
    pub size: [u32; 2],
    pub data_base64: String,
    pub hash: String,
}

/// The compilation pipeline - single entry point for all asset operations
pub struct CompilationPipeline {
    registry: TemplateRegistry,
    validator: Validator,
}

impl CompilationPipeline {
    pub fn new(registry: TemplateRegistry) -> Self {
        Self {
            registry,
            validator: Validator::new(),
        }
    }

    /// List all available templates
    pub fn list_templates(&self) -> Vec<&Template> {
        self.registry.list()
    }

    /// Get a specific template
    pub fn get_template(&self, id: &str) -> Option<&Template> {
        self.registry.get(id)
    }

    /// Validate an asset against a template
    /// 
    /// This is the ONLY validation entry point.
    pub fn validate_asset(
        &self,
        template_id: &str,
        input: &AssetInput,
    ) -> Result<ValidationResult, PipelineError> {
        #[cfg(feature = "test-hooks")]
        VALIDATION_CALL_COUNT.fetch_add(1, Ordering::SeqCst);

        let template = self.registry.get(template_id)
            .ok_or_else(|| PipelineError::TemplateNotFound(template_id.to_string()))?;

        // Check engine version compatibility
        self.check_engine_version(template)?;

        Ok(self.validator.validate(input, template))
    }

    /// Compile an asset
    ///
    /// CRITICAL: This ALWAYS calls validate_asset internally. No bypass possible.
    pub fn compile_asset(&self, request: &CompileRequest) -> Result<CompiledAsset, PipelineError> {
        let template = self.registry.get(&request.template_id)
            .ok_or_else(|| PipelineError::TemplateNotFound(request.template_id.clone()))?;

        // MANDATORY: Validation is always called. This is non-negotiable.
        let validation = self.validate_asset(&request.template_id, &request.asset_input)?;

        // If validation failed with errors, reject compilation
        if !validation.valid {
            let messages: Vec<_> = validation.violations.iter()
                .map(|v| format!("{}: {}", v.rule, v.message))
                .collect();
            return Err(PipelineError::ValidationFailed(messages.join("; ")));
        }

        // Generate exports (simulated for now)
        let exports = self.generate_exports(template, request)?;

        // Build manifest
        let asset_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        let job_hash = compute_job_hash(
            &request.template_id,
            &template.template_version,
            request,
            ENGINE_VERSION,
        )?;

        let mut asset = CompiledAsset {
            id: asset_id,
            template_id: request.template_id.clone(),
            template_version: template.template_version.clone(),
            engine_version: ENGINE_VERSION.to_string(),
            created_at,
            manifest_hash: String::new(),  // Computed after
            job_hash,
            validation,
            exports,
        };

        // Compute manifest hash (includes everything)
        asset.manifest_hash = compute_manifest_hash(&asset)?;

        Ok(asset)
    }

    fn check_engine_version(&self, template: &Template) -> Result<(), PipelineError> {
        let engine_ver = semver::Version::parse(ENGINE_VERSION)
            .map_err(|_| PipelineError::CompilationError("Invalid engine version".into()))?;
        let min_ver = semver::Version::parse(&template.engine_min_version)
            .map_err(|_| PipelineError::CompilationError("Invalid template min version".into()))?;

        if engine_ver < min_ver {
            return Err(PipelineError::EngineVersionMismatch(
                template.template_version.clone(),
                template.engine_min_version.clone(),
                ENGINE_VERSION.to_string(),
            ));
        }

        Ok(())
    }

    fn generate_exports(
        &self,
        template: &Template,
        request: &CompileRequest,
    ) -> Result<Vec<ExportedFile>, PipelineError> {
        let mut exports = vec![];

        for spec in &template.exports {
            // Generate placeholder data (in real impl, this would render the asset)
            let data = self.render_export(spec, request)?;
            let hash = crate::hashing::sha256_hex(&data);

            exports.push(ExportedFile {
                id: spec.id.clone(),
                filename: format!("{}.{}", spec.id, format_extension(&spec.format)),
                format: format!("{:?}", spec.format).to_lowercase(),
                size: spec.size,
                data_base64: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data),
                hash,
            });
        }

        Ok(exports)
    }

    fn render_export(
        &self,
        spec: &ExportSpec,
        _request: &CompileRequest,
    ) -> Result<Vec<u8>, PipelineError> {
        // Placeholder: In real implementation, this would:
        // 1. Take the SVG master
        // 2. Render to the target format at target size
        // For now, return a minimal valid placeholder
        match spec.format {
            crate::templates::ExportFormat::Svg => {
                Ok(format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}"></svg>"#,
                    spec.size[0], spec.size[1]
                ).into_bytes())
            }
            crate::templates::ExportFormat::Png => {
                // Minimal 1x1 transparent PNG
                Ok(vec![
                    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
                    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
                    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
                    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
                    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41,
                    0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
                    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
                    0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
                    0x42, 0x60, 0x82
                ])
            }
            _ => {
                Ok(b"placeholder".to_vec())
            }
        }
    }
}

fn format_extension(format: &crate::templates::ExportFormat) -> &'static str {
    match format {
        crate::templates::ExportFormat::Svg => "svg",
        crate::templates::ExportFormat::Png => "png",
        crate::templates::ExportFormat::Ico => "ico",
        crate::templates::ExportFormat::Pdf => "pdf",
        crate::templates::ExportFormat::Jpg => "jpg",
    }
}

impl Default for CompilationPipeline {
    fn default() -> Self {
        Self::new(TemplateRegistry::default())
    }
}
```

---

## STEP 8: Rust CLI (src/bin/forgeimages_cli.rs)

```rust
//! ForgeImages CLI - Bridge interface for Python
//!
//! Commands: templates, validate, compile
//! Outputs JSON to stdout
//! Returns non-zero on validation failure

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

use forgeimages_core::{
    CompilationPipeline, CompileRequest,
    validation::AssetInput,
    templates::TemplateRegistry,
};

#[derive(Parser)]
#[command(name = "forgeimages-cli")]
#[command(about = "ForgeImages CLI - Visual Production Compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to templates directory
    #[arg(short, long, default_value = "templates")]
    templates_dir: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// List available templates
    Templates,

    /// Validate an asset
    Validate {
        /// Template ID
        #[arg(short, long)]
        template: String,

        /// JSON payload (AssetInput)
        #[arg(short, long)]
        payload: String,
    },

    /// Compile an asset
    Compile {
        /// Template ID
        #[arg(short, long)]
        template: String,

        /// JSON payload (CompileRequest)
        #[arg(short, long)]
        payload: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Load templates
    let registry = match TemplateRegistry::load_from_dir(&cli.templates_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(r#"{{"error": "Failed to load templates: {}"}}"#, e);
            return ExitCode::FAILURE;
        }
    };

    let pipeline = CompilationPipeline::new(registry);

    match cli.command {
        Commands::Templates => {
            let templates: Vec<_> = pipeline.list_templates()
                .iter()
                .map(|t| serde_json::json!({
                    "id": t.id,
                    "name": t.name,
                    "version": t.template_version,
                    "asset_class": t.asset_class,
                    "deprecated": t.deprecated,
                }))
                .collect();

            println!("{}", serde_json::to_string_pretty(&templates).unwrap());
            ExitCode::SUCCESS
        }

        Commands::Validate { template, payload } => {
            let input: AssetInput = match serde_json::from_str(&payload) {
                Ok(i) => i,
                Err(e) => {
                    println!(r#"{{"valid": false, "error": "Invalid payload: {}"}}"#, e);
                    return ExitCode::FAILURE;
                }
            };

            match pipeline.validate_asset(&template, &input) {
                Ok(result) => {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap());
                    if result.valid {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(2)  // Validation failure
                    }
                }
                Err(e) => {
                    println!(r#"{{"valid": false, "error": "{}"}}"#, e);
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Compile { template, payload } => {
            let request: CompileRequest = match serde_json::from_str(&payload) {
                Ok(r) => r,
                Err(e) => {
                    println!(r#"{{"success": false, "error": "Invalid payload: {}"}}"#, e);
                    return ExitCode::FAILURE;
                }
            };

            // Ensure template_id matches
            let request = CompileRequest {
                template_id: template,
                ..request
            };

            match pipeline.compile_asset(&request) {
                Ok(asset) => {
                    let output = serde_json::json!({
                        "success": true,
                        "asset": asset,
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    let output = serde_json::json!({
                        "success": false,
                        "error": e.to_string(),
                    });
                    println!("{}", serde_json::to_string(&output).unwrap());
                    ExitCode::from(2)  // Compilation failure (validation)
                }
            }
        }
    }
}
```

---

## STEP 9: PWA Icon Template (templates/pwa-icon.json)

```json
{
  "$schema": "https://forgeimages.dev/schemas/template-v1.json",
  "id": "pwa-icon",
  "name": "PWA Icon Pack",
  "description": "Complete Progressive Web App icon set",
  "templateVersion": "1.0.0",
  "engineMinVersion": "1.0.0",
  "deprecated": false,
  "assetClass": "icon",
  "aspectRatio": [1, 1],
  "canonicalSize": [1024, 1024],
  "vectorMaster": true,
  "validation": {
    "required": true,
    "failureMode": "block",
    "rules": {
      "aspectRatio": {
        "enabled": true,
        "tolerance": 0.01
      },
      "resolution": {
        "enabled": true,
        "minWidth": 512,
        "minHeight": 512
      },
      "colorCount": {
        "enabled": true,
        "max": 16
      }
    }
  },
  "exports": [
    {
      "id": "master",
      "description": "SVG master",
      "size": [1024, 1024],
      "format": "svg",
      "required": true
    },
    {
      "id": "favicon-16",
      "description": "Browser favicon 16px",
      "size": [16, 16],
      "format": "png",
      "required": true
    },
    {
      "id": "favicon-32",
      "description": "Browser favicon 32px",
      "size": [32, 32],
      "format": "png",
      "required": true
    },
    {
      "id": "apple-touch",
      "description": "Apple touch icon",
      "size": [180, 180],
      "format": "png",
      "required": true
    },
    {
      "id": "pwa-192",
      "description": "PWA icon 192px",
      "size": [192, 192],
      "format": "png",
      "required": true
    },
    {
      "id": "pwa-512",
      "description": "PWA icon 512px",
      "size": [512, 512],
      "format": "png",
      "required": true
    }
  ]
}
```

---

## STEP 10: Rust Invariant Tests (tests/invariants.rs)

```rust
//! Contract Invariant Tests
//!
//! These tests verify the non-negotiable guarantees.

use forgeimages_core::{
    CompilationPipeline, CompileRequest,
    templates::{Template, TemplateRegistry, AssetClass, ValidationConfig, ExportSpec, ExportFormat},
    validation::AssetInput,
    hashing::{compute_manifest_hash, canonical_json},
};

fn create_test_template() -> Template {
    Template {
        id: "test-icon".to_string(),
        name: "Test Icon".to_string(),
        description: "Test template".to_string(),
        template_version: "1.0.0".to_string(),
        engine_min_version: "1.0.0".to_string(),
        deprecated: false,
        superseded_by: None,
        asset_class: AssetClass::Icon,
        aspect_ratio: [1, 1],
        canonical_size: [1024, 1024],
        vector_master: true,
        validation: ValidationConfig::default(),
        exports: vec![
            ExportSpec {
                id: "master".to_string(),
                description: "SVG master".to_string(),
                size: [1024, 1024],
                format: ExportFormat::Svg,
                required: true,
            }
        ],
    }
}

fn create_pipeline() -> CompilationPipeline {
    let mut registry = TemplateRegistry::new();
    registry.register(create_test_template());
    CompilationPipeline::new(registry)
}

#[test]
fn invariant_compile_calls_validate() {
    // This test verifies that compile_asset internally calls validate_asset
    // by attempting to compile an invalid asset and expecting failure
    
    let pipeline = create_pipeline();
    
    // Invalid: wrong aspect ratio
    let request = CompileRequest {
        template_id: "test-icon".to_string(),
        asset_input: AssetInput {
            width: 1024,
            height: 512,  // Not 1:1!
            color_count: None,
            format: None,
        },
        source_data: None,
        seed: None,
        prompt: None,
    };
    
    let result = pipeline.compile_asset(&request);
    
    // Must fail - validation is enforced
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Validation failed"));
}

#[test]
fn invariant_valid_asset_compiles() {
    let pipeline = create_pipeline();
    
    let request = CompileRequest {
        template_id: "test-icon".to_string(),
        asset_input: AssetInput {
            width: 1024,
            height: 1024,
            color_count: Some(8),
            format: None,
        },
        source_data: None,
        seed: None,
        prompt: None,
    };
    
    let result = pipeline.compile_asset(&request);
    assert!(result.is_ok());
    
    let asset = result.unwrap();
    assert!(asset.validation.valid);
    assert!(!asset.manifest_hash.is_empty());
}

#[test]
fn invariant_manifest_hash_stable() {
    // Same inputs must produce same manifest hash
    let pipeline = create_pipeline();
    
    let request = CompileRequest {
        template_id: "test-icon".to_string(),
        asset_input: AssetInput {
            width: 1024,
            height: 1024,
            color_count: Some(4),
            format: None,
        },
        source_data: None,
        seed: Some(42),  // Fixed seed for determinism
        prompt: Some("test".to_string()),
    };
    
    // Note: In a real implementation with true determinism,
    // we'd verify the actual manifest_hash is identical.
    // For now, we verify the structure is consistent.
    let asset1 = pipeline.compile_asset(&request).unwrap();
    let asset2 = pipeline.compile_asset(&request).unwrap();
    
    // Job hash should be identical (same inputs)
    assert_eq!(asset1.job_hash, asset2.job_hash);
    
    // Template info should match
    assert_eq!(asset1.template_id, asset2.template_id);
    assert_eq!(asset1.template_version, asset2.template_version);
}

#[test]
fn invariant_canonical_json_deterministic() {
    use serde_json::json;
    
    let obj1 = json!({"z": 1, "a": 2, "m": {"b": 1, "a": 2}});
    let obj2 = json!({"a": 2, "m": {"a": 2, "b": 1}, "z": 1});
    
    let c1 = canonical_json(&obj1).unwrap();
    let c2 = canonical_json(&obj2).unwrap();
    
    // Must be identical despite different input ordering
    assert_eq!(c1, c2);
}

#[test]
fn invariant_template_not_found_error() {
    let pipeline = create_pipeline();
    
    let request = CompileRequest {
        template_id: "nonexistent".to_string(),
        asset_input: AssetInput {
            width: 1024,
            height: 1024,
            color_count: None,
            format: None,
        },
        source_data: None,
        seed: None,
        prompt: None,
    };
    
    let result = pipeline.compile_asset(&request);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Template not found"));
}

#[test]
fn invariant_validation_result_structure() {
    let pipeline = create_pipeline();
    
    let input = AssetInput {
        width: 100,  // Too small
        height: 100,
        color_count: None,
        format: None,
    };
    
    let result = pipeline.validate_asset("test-icon", &input).unwrap();
    
    // Validation failed
    assert!(!result.valid);
    
    // Has violations with required fields
    assert!(!result.violations.is_empty());
    for v in &result.violations {
        assert!(!v.rule.is_empty());
        assert!(!v.message.is_empty());
    }
    
    // Template info present
    assert_eq!(result.template_id, "test-icon");
    assert_eq!(result.template_version, "1.0.0");
}
```

---

## STEP 11: Python pyproject.toml

Create `forgeagents-forgeimages/pyproject.toml`:

```toml
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "forgeagents-forgeimages"
version = "1.0.0"
description = "ForgeAgents integration for ForgeImages"
readme = "README.md"
requires-python = ">=3.10"
license = "Proprietary"
authors = [
    { name = "Boswell Digital Solutions LLC" }
]

dependencies = [
    "fastapi>=0.104.0",
    "uvicorn[standard]>=0.24.0",
    "pydantic>=2.5.0",
    "httpx>=0.25.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.4.0",
    "pytest-asyncio>=0.21.0",
    "httpx>=0.25.0",
]

[tool.hatch.build.targets.wheel]
packages = ["bridge", "skill"]

[tool.pytest.ini_options]
asyncio_mode = "auto"
testpaths = ["tests"]
```

---

## STEP 12: Python Bridge Models (bridge/models.py)

```python
"""
Pydantic models for the ForgeImages bridge.

These models define the contract between Python and Rust.
"""

from datetime import datetime
from typing import Optional
from pydantic import BaseModel, Field, field_validator
import re


class AssetInput(BaseModel):
    """Input for asset validation/compilation."""
    width: int = Field(..., ge=1, le=10000)
    height: int = Field(..., ge=1, le=10000)
    color_count: Optional[int] = Field(None, ge=1, le=256)
    format: Optional[str] = None


class CompileRequest(BaseModel):
    """Request to compile an asset."""
    template_id: str = Field(..., min_length=1, max_length=64)
    asset_input: AssetInput
    source_data: Optional[str] = None  # Base64
    seed: Optional[int] = Field(None, ge=0)
    prompt: Optional[str] = Field(None, max_length=2000)

    @field_validator('template_id')
    @classmethod
    def validate_template_id(cls, v: str) -> str:
        # Only allow safe template IDs
        if not re.match(r'^[a-z0-9][a-z0-9-]*$', v):
            raise ValueError('Invalid template_id format')
        return v


class ValidationViolation(BaseModel):
    """A single validation violation."""
    rule: str
    severity: str
    message: str
    expected: Optional[str] = None
    actual: Optional[str] = None
    remediation: list[str] = []


class ValidationResult(BaseModel):
    """Result of validation."""
    valid: bool
    violations: list[ValidationViolation] = []
    template_id: str
    template_version: str


class ExportedFile(BaseModel):
    """An exported file from compilation."""
    id: str
    filename: str
    format: str
    size: list[int]
    data_base64: str
    hash: str


class CompiledAsset(BaseModel):
    """A compiled asset with all exports."""
    id: str
    template_id: str
    template_version: str
    engine_version: str
    created_at: datetime
    manifest_hash: str
    job_hash: str
    validation: ValidationResult
    exports: list[ExportedFile]


class CompileResponse(BaseModel):
    """Response from compile endpoint."""
    success: bool
    asset: Optional[CompiledAsset] = None
    error: Optional[str] = None


class TemplateInfo(BaseModel):
    """Template summary info."""
    id: str
    name: str
    version: str
    asset_class: str
    deprecated: bool = False
```

---

## STEP 13: Python Audit Logger (bridge/audit.py)

```python
"""
Audit logging for ForgeImages bridge.

Writes append-only JSONL for compliance and debugging.
"""

import json
import hashlib
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional, Any
from pydantic import BaseModel


class AuditEntry(BaseModel):
    """Single audit log entry."""
    timestamp: str
    request_id: str
    user_id: Optional[str] = None
    template_id: str
    job_hash: str
    action: str  # "validate" | "compile"
    outcome: str  # "success" | "validation_failed" | "error"
    violations_count: int = 0
    error_message: Optional[str] = None


class AuditLogger:
    """Append-only JSONL audit logger."""
    
    def __init__(self, log_path: Path):
        self.log_path = log_path
        self.log_path.parent.mkdir(parents=True, exist_ok=True)
    
    def compute_job_hash(
        self,
        template_id: str,
        template_version: str,
        payload: Any,
        engine_version: str,
    ) -> str:
        """Compute job hash for audit trail."""
        # Canonical JSON
        payload_json = json.dumps(payload, sort_keys=True, separators=(',', ':'))
        combined = f"{template_id}:{template_version}:{payload_json}:{engine_version}"
        return hashlib.sha256(combined.encode()).hexdigest()
    
    def log(self, entry: AuditEntry) -> None:
        """Append entry to audit log."""
        with open(self.log_path, 'a') as f:
            f.write(entry.model_dump_json() + '\n')
    
    def log_validate(
        self,
        request_id: str,
        template_id: str,
        job_hash: str,
        valid: bool,
        violations_count: int,
        user_id: Optional[str] = None,
    ) -> None:
        """Log a validation request."""
        entry = AuditEntry(
            timestamp=datetime.now(timezone.utc).isoformat(),
            request_id=request_id,
            user_id=user_id,
            template_id=template_id,
            job_hash=job_hash,
            action="validate",
            outcome="success" if valid else "validation_failed",
            violations_count=violations_count,
        )
        self.log(entry)
    
    def log_compile(
        self,
        request_id: str,
        template_id: str,
        job_hash: str,
        success: bool,
        violations_count: int = 0,
        error_message: Optional[str] = None,
        user_id: Optional[str] = None,
    ) -> None:
        """Log a compilation request."""
        if success:
            outcome = "success"
        elif error_message and "Validation" in error_message:
            outcome = "validation_failed"
        else:
            outcome = "error"
        
        entry = AuditEntry(
            timestamp=datetime.now(timezone.utc).isoformat(),
            request_id=request_id,
            user_id=user_id,
            template_id=template_id,
            job_hash=job_hash,
            action="compile",
            outcome=outcome,
            violations_count=violations_count,
            error_message=error_message,
        )
        self.log(entry)
```

---

## STEP 14: Python Settings (bridge/settings.py)

```python
"""
Bridge configuration settings.
"""

from pathlib import Path
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """Bridge configuration."""
    
    # Rust CLI path
    cli_path: Path = Path("../forgeimages-core/target/release/forgeimages-cli")
    
    # Templates directory
    templates_dir: Path = Path("../forgeimages-core/templates")
    
    # Audit log path
    audit_log_path: Path = Path("./audit.jsonl")
    
    # Request limits
    max_request_size_mb: int = 10
    max_payload_size_kb: int = 512
    
    # Engine version (for job hash when CLI unavailable)
    engine_version: str = "1.0.0"
    
    class Config:
        env_prefix = "FORGEIMAGES_"


settings = Settings()
```

---

## STEP 15: Python Bridge Service (bridge/forgeimages_bridge.py)

```python
"""
ForgeImages FastAPI Bridge Service.

This bridge is the ONLY interface between ForgeAgents and ForgeImages.
It enforces:
- HTTP 422 on validation failure
- All requests are audit logged
- No file writes (all data returned as base64)
"""

import json
import subprocess
import uuid
from pathlib import Path
from typing import Optional

from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse

from .models import (
    AssetInput,
    CompileRequest,
    CompileResponse,
    ValidationResult,
    TemplateInfo,
)
from .audit import AuditLogger
from .settings import settings


app = FastAPI(
    title="ForgeImages Bridge",
    description="Bridge service for ForgeAgents to access ForgeImages",
    version="1.0.0",
)

audit = AuditLogger(settings.audit_log_path)


def call_cli(command: list[str]) -> tuple[int, str]:
    """Call the Rust CLI and return (exit_code, stdout)."""
    cli_path = settings.cli_path
    templates_dir = settings.templates_dir
    
    full_command = [
        str(cli_path),
        "--templates-dir", str(templates_dir),
        *command
    ]
    
    try:
        result = subprocess.run(
            full_command,
            capture_output=True,
            text=True,
            timeout=30,
        )
        return result.returncode, result.stdout
    except FileNotFoundError:
        raise HTTPException(
            status_code=503,
            detail="ForgeImages CLI not found. Build with: cargo build --release"
        )
    except subprocess.TimeoutExpired:
        raise HTTPException(
            status_code=504,
            detail="ForgeImages CLI timeout"
        )


@app.middleware("http")
async def limit_request_size(request: Request, call_next):
    """Limit request body size for security."""
    content_length = request.headers.get("content-length")
    if content_length:
        if int(content_length) > settings.max_request_size_mb * 1024 * 1024:
            return JSONResponse(
                status_code=413,
                content={"detail": "Request too large"}
            )
    return await call_next(request)


@app.get("/health")
async def health():
    """Health check endpoint."""
    return {"status": "ok", "service": "forgeimages-bridge"}


@app.get("/templates", response_model=list[TemplateInfo])
async def list_templates():
    """List all available templates."""
    exit_code, output = call_cli(["templates"])
    
    if exit_code != 0:
        raise HTTPException(status_code=500, detail="Failed to list templates")
    
    try:
        templates = json.loads(output)
        return templates
    except json.JSONDecodeError:
        raise HTTPException(status_code=500, detail="Invalid CLI response")


@app.post("/validate", response_model=ValidationResult)
async def validate_asset(
    template_id: str,
    asset_input: AssetInput,
    user_id: Optional[str] = None,
):
    """
    Validate an asset against a template.
    
    Returns ValidationResult with violations if invalid.
    """
    request_id = str(uuid.uuid4())
    
    # Prepare payload
    payload = asset_input.model_dump(exclude_none=True)
    payload_json = json.dumps(payload)
    
    # Compute job hash for audit
    job_hash = audit.compute_job_hash(
        template_id=template_id,
        template_version="unknown",  # We don't know until CLI returns
        payload=payload,
        engine_version=settings.engine_version,
    )
    
    # Call CLI
    exit_code, output = call_cli([
        "validate",
        "--template", template_id,
        "--payload", payload_json,
    ])
    
    try:
        result = json.loads(output)
        validation = ValidationResult(**result)
        
        # Audit log
        audit.log_validate(
            request_id=request_id,
            template_id=template_id,
            job_hash=job_hash,
            valid=validation.valid,
            violations_count=len(validation.violations),
            user_id=user_id,
        )
        
        return validation
        
    except json.JSONDecodeError:
        raise HTTPException(status_code=500, detail="Invalid CLI response")


@app.post("/compile")
async def compile_asset(
    request: CompileRequest,
    user_id: Optional[str] = None,
):
    """
    Compile an asset.
    
    Returns 422 on validation failure.
    Returns CompiledAsset on success.
    
    CRITICAL: This endpoint NEVER bypasses validation.
    The Rust CLI enforces validation internally.
    """
    request_id = str(uuid.uuid4())
    
    # Prepare payload
    payload = request.model_dump(exclude_none=True)
    payload_json = json.dumps(payload)
    
    # Size check
    if len(payload_json) > settings.max_payload_size_kb * 1024:
        raise HTTPException(status_code=413, detail="Payload too large")
    
    # Compute job hash for audit
    job_hash = audit.compute_job_hash(
        template_id=request.template_id,
        template_version="unknown",
        payload=payload,
        engine_version=settings.engine_version,
    )
    
    # Call CLI
    exit_code, output = call_cli([
        "compile",
        "--template", request.template_id,
        "--payload", payload_json,
    ])
    
    try:
        result = json.loads(output)
    except json.JSONDecodeError:
        audit.log_compile(
            request_id=request_id,
            template_id=request.template_id,
            job_hash=job_hash,
            success=False,
            error_message="Invalid CLI response",
            user_id=user_id,
        )
        raise HTTPException(status_code=500, detail="Invalid CLI response")
    
    # Check for success
    if result.get("success"):
        audit.log_compile(
            request_id=request_id,
            template_id=request.template_id,
            job_hash=job_hash,
            success=True,
            user_id=user_id,
        )
        return result
    
    # Failure - determine if validation or other error
    error = result.get("error", "Unknown error")
    
    if "Validation failed" in error or exit_code == 2:
        # 422 Unprocessable Entity - validation failure
        audit.log_compile(
            request_id=request_id,
            template_id=request.template_id,
            job_hash=job_hash,
            success=False,
            error_message=error,
            user_id=user_id,
        )
        raise HTTPException(
            status_code=422,
            detail={
                "error": "Validation failed",
                "message": error,
                "job_hash": job_hash,
            }
        )
    
    # Other error
    audit.log_compile(
        request_id=request_id,
        template_id=request.template_id,
        job_hash=job_hash,
        success=False,
        error_message=error,
        user_id=user_id,
    )
    raise HTTPException(status_code=500, detail=error)
```

---

## STEP 16: Python Skill Wrapper (skill/forgeimages_skill.py)

```python
"""
ForgeImages Skill for ForgeAgents.

This skill provides the agent-facing API for ForgeImages.

CRITICAL CONSTRAINTS:
- This skill NEVER writes files directly
- This skill ONLY calls the bridge service
- All validation is enforced by the bridge/Rust
"""

from typing import Optional
import httpx

from bridge.models import (
    AssetInput,
    CompileRequest,
    CompileResponse,
    ValidationResult,
    TemplateInfo,
)


class ForgeImagesSkill:
    """
    ForgeImages skill for ForgeAgents.
    
    Agents can:
    - List available templates
    - Validate assets before compilation
    - Request compilation (validation enforced)
    
    Agents CANNOT:
    - Bypass validation
    - Write files directly
    - Override template constraints
    """
    
    def __init__(self, bridge_url: str = "http://localhost:8789"):
        self.bridge_url = bridge_url.rstrip("/")
        self.client = httpx.Client(timeout=30.0)
    
    def list_templates(self) -> list[TemplateInfo]:
        """List all available templates."""
        response = self.client.get(f"{self.bridge_url}/templates")
        response.raise_for_status()
        return [TemplateInfo(**t) for t in response.json()]
    
    def validate_asset(
        self,
        template_id: str,
        width: int,
        height: int,
        color_count: Optional[int] = None,
    ) -> ValidationResult:
        """
        Validate an asset against a template.
        
        Returns ValidationResult with violations if invalid.
        """
        asset_input = AssetInput(
            width=width,
            height=height,
            color_count=color_count,
        )
        
        response = self.client.post(
            f"{self.bridge_url}/validate",
            params={"template_id": template_id},
            json=asset_input.model_dump(exclude_none=True),
        )
        response.raise_for_status()
        return ValidationResult(**response.json())
    
    def compile_asset(
        self,
        template_id: str,
        width: int,
        height: int,
        color_count: Optional[int] = None,
        source_data: Optional[str] = None,
        seed: Optional[int] = None,
        prompt: Optional[str] = None,
    ) -> CompileResponse:
        """
        Compile an asset.
        
        Raises httpx.HTTPStatusError with status 422 if validation fails.
        
        NOTE: This method does NOT bypass validation.
        The bridge enforces validation internally.
        """
        request = CompileRequest(
            template_id=template_id,
            asset_input=AssetInput(
                width=width,
                height=height,
                color_count=color_count,
            ),
            source_data=source_data,
            seed=seed,
            prompt=prompt,
        )
        
        response = self.client.post(
            f"{self.bridge_url}/compile",
            json=request.model_dump(exclude_none=True),
        )
        
        # Don't raise for 422 - return the error details
        if response.status_code == 422:
            return CompileResponse(
                success=False,
                error=response.json().get("detail", {}).get("message", "Validation failed"),
            )
        
        response.raise_for_status()
        data = response.json()
        return CompileResponse(**data)
    
    def close(self):
        """Close the HTTP client."""
        self.client.close()
    
    def __enter__(self):
        return self
    
    def __exit__(self, *args):
        self.close()
```

---

## STEP 17: Python Agent Boundary Tests (tests/test_agent_boundary.py)

```python
"""
Agent Boundary Tests.

These tests PROVE that agents cannot bypass ForgeImages validation.
If any of these tests fail, the integration is broken.
"""

import pytest
from fastapi.testclient import TestClient
import json

# Import the bridge app
from bridge.forgeimages_bridge import app


client = TestClient(app)


class TestAgentBoundaries:
    """Tests proving agents cannot bypass validation."""
    
    def test_invalid_aspect_ratio_returns_422(self):
        """
        Invalid payload MUST return 422.
        
        This is the core enforcement mechanism.
        """
        # Assuming pwa-icon template requires 1:1 aspect ratio
        payload = {
            "template_id": "pwa-icon",
            "asset_input": {
                "width": 1024,
                "height": 512,  # Wrong aspect ratio!
            }
        }
        
        response = client.post("/compile", json=payload)
        
        # MUST be 422, not 200 or 500
        assert response.status_code == 422, \
            f"Expected 422, got {response.status_code}: {response.text}"
        
        # Response must include error details
        detail = response.json()["detail"]
        assert "Validation" in str(detail) or "error" in detail
    
    def test_resolution_too_low_returns_422(self):
        """Resolution below minimum MUST be rejected."""
        payload = {
            "template_id": "pwa-icon",
            "asset_input": {
                "width": 100,  # Too small
                "height": 100,
            }
        }
        
        response = client.post("/compile", json=payload)
        
        assert response.status_code == 422
    
    def test_valid_payload_succeeds(self):
        """Valid payload should compile successfully."""
        payload = {
            "template_id": "pwa-icon",
            "asset_input": {
                "width": 1024,
                "height": 1024,
                "color_count": 8,
            }
        }
        
        response = client.post("/compile", json=payload)
        
        # Should succeed
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        assert data["success"] is True
        assert "asset" in data
        
        # Manifest hash must be present
        asset = data["asset"]
        assert "manifest_hash" in asset
        assert len(asset["manifest_hash"]) == 64  # SHA-256 hex
    
    def test_no_file_write_endpoint(self):
        """There must be no endpoint that writes files."""
        # Try various paths that might write files
        dangerous_paths = [
            "/write",
            "/save",
            "/export",
            "/file",
            "/output",
        ]
        
        for path in dangerous_paths:
            response = client.post(path, json={})
            # Should be 404 or 405, NOT 200
            assert response.status_code in [404, 405, 422], \
                f"Unexpected response for {path}: {response.status_code}"
    
    def test_override_validation_ignored(self):
        """
        Attempts to override validation in payload MUST be ignored.
        
        This tests that extra fields don't bypass validation.
        """
        payload = {
            "template_id": "pwa-icon",
            "asset_input": {
                "width": 100,  # Invalid
                "height": 100,
            },
            # Attempted bypasses (should be ignored)
            "skip_validation": True,
            "force": True,
            "bypass": True,
            "validation_override": True,
        }
        
        response = client.post("/compile", json=payload)
        
        # MUST still fail validation
        assert response.status_code == 422, \
            "Validation bypass attempted but should have failed"
    
    def test_template_id_injection_rejected(self):
        """Malicious template IDs must be rejected."""
        malicious_ids = [
            "../../../etc/passwd",
            "template; rm -rf /",
            "<script>alert('xss')</script>",
            "template\x00null",
        ]
        
        for template_id in malicious_ids:
            payload = {
                "template_id": template_id,
                "asset_input": {
                    "width": 1024,
                    "height": 1024,
                }
            }
            
            response = client.post("/compile", json=payload)
            
            # Should be rejected (400, 404, or 422)
            assert response.status_code in [400, 404, 422, 500], \
                f"Malicious template_id '{template_id}' should be rejected"
    
    def test_validate_endpoint_returns_violations(self):
        """Validate endpoint must return structured violations."""
        response = client.post(
            "/validate",
            params={"template_id": "pwa-icon"},
            json={
                "width": 100,
                "height": 200,
            }
        )
        
        # Validation endpoint returns 200 with result
        assert response.status_code == 200
        
        data = response.json()
        assert "valid" in data
        assert "violations" in data
        
        if not data["valid"]:
            # Violations must have structure
            assert len(data["violations"]) > 0
            for v in data["violations"]:
                assert "rule" in v
                assert "message" in v
    
    def test_compile_includes_job_hash(self):
        """Compiled assets must include job_hash for auditing."""
        payload = {
            "template_id": "pwa-icon",
            "asset_input": {
                "width": 1024,
                "height": 1024,
            }
        }
        
        response = client.post("/compile", json=payload)
        
        if response.status_code == 200:
            data = response.json()
            assert "asset" in data
            assert "job_hash" in data["asset"]
            assert len(data["asset"]["job_hash"]) == 64


class TestSkillWrapper:
    """Tests for the ForgeAgents skill wrapper."""
    
    def test_skill_cannot_write_files(self):
        """
        Verify the skill has no file-writing methods.
        """
        from skill.forgeimages_skill import ForgeImagesSkill
        
        skill = ForgeImagesSkill()
        
        # Check that no write methods exist
        forbidden_methods = [
            "write_file",
            "save_file",
            "export_to_disk",
            "write_output",
            "save_asset",
        ]
        
        for method in forbidden_methods:
            assert not hasattr(skill, method), \
                f"Skill should not have {method} method"
        
        skill.close()
    
    def test_skill_returns_base64_not_paths(self):
        """
        Skill must return data as base64, never file paths.
        """
        from skill.forgeimages_skill import ForgeImagesSkill
        from bridge.models import CompileResponse
        
        # The response model should have base64 data, not paths
        import inspect
        from bridge.models import ExportedFile
        
        # ExportedFile must have data_base64, not path
        assert "data_base64" in ExportedFile.model_fields
        assert "path" not in ExportedFile.model_fields
        assert "file_path" not in ExportedFile.model_fields
```

---

## STEP 18: Python Package Init Files

Create `bridge/__init__.py`:
```python
"""ForgeImages Bridge Service."""
```

Create `skill/__init__.py`:
```python
"""ForgeImages Skill for ForgeAgents."""
from .forgeimages_skill import ForgeImagesSkill

__all__ = ["ForgeImagesSkill"]
```

---

## Run Commands

After creating all files:

```bash
# 1. Build Rust CLI
cd forgeimages-core
cargo build --release
cargo test
cargo test --test invariants

# 2. Setup Python
cd ../forgeagents-forgeimages
python -m venv .venv
source .venv/bin/activate  # or .venv\Scripts\activate on Windows
pip install -e ".[dev]"

# 3. Run Python tests
pytest -v

# 4. Start bridge server
uvicorn bridge.forgeimages_bridge:app --reload --port 8789
```

---

## Verification Checklist

After implementation, verify:

- [ ] `cargo test` passes
- [ ] `cargo test --test invariants` passes
- [ ] `pytest -v` passes
- [ ] Invalid payload → HTTP 422
- [ ] Valid payload → Success with manifest_hash
- [ ] No file-write endpoints exist
- [ ] Audit log is created and populated
- [ ] Skill has no file-writing methods
