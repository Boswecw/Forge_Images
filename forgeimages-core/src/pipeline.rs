//! Compilation Pipeline - Single Entry Point
//!
//! CRITICAL: compile_asset MUST call validate internally. No bypass.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::templates::{Template, TemplateRegistry};
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

    #[error("Invalid source data: {0}")]
    InvalidSourceData(String),

    #[error("Render error: {0}")]
    RenderError(#[from] crate::render::RenderError),
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
        // Resolve the SVG vector master once (Law 1: SVG Is Truth). Every raster
        // export is deterministically derived from this single master.
        let master_svg = self.resolve_master_svg(template, request)?;

        let mut exports = vec![];

        for spec in &template.exports {
            let data = crate::render::render(
                &master_svg,
                &spec.format,
                spec.size[0],
                spec.size[1],
            )?;
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

    /// Resolve the SVG vector master for a compile request.
    ///
    /// If the caller supplies `source_data` (base64-encoded SVG), that is the
    /// master. Otherwise a deterministic default master is synthesized from the
    /// template's canonical size — keeping compilation total and reproducible
    /// even without a supplied source. The engine never *generates* imagery;
    /// AI/generative production happens upstream (NeuroForge) and arrives here
    /// as an SVG to be deterministically compiled.
    fn resolve_master_svg(
        &self,
        template: &Template,
        request: &CompileRequest,
    ) -> Result<String, PipelineError> {
        match &request.source_data {
            Some(b64) => {
                let bytes = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    b64,
                )
                .map_err(|e| PipelineError::InvalidSourceData(e.to_string()))?;
                String::from_utf8(bytes)
                    .map_err(|e| PipelineError::InvalidSourceData(e.to_string()))
            }
            None => Ok(crate::render::default_master_svg(
                template.canonical_size[0],
                template.canonical_size[1],
            )),
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
