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
