# Step 1: Rust Core Crate

Create the ForgeImages core Rust crate with all modules.

## Directory Structure

```
forgeimages-core/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── templates.rs
│   ├── validation.rs
│   ├── hashing.rs
│   ├── print.rs
│   └── pipeline.rs
├── templates/
│   └── pwa-icon.json
└── tests/
    └── invariants.rs
```

## Cargo.toml

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

## src/lib.rs

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

## Key Design Decisions

1. **semver crate** - Real version comparison, not string matching
2. **sha2 crate** - SHA-256 for manifest hashes
3. **thiserror** - Clean error handling
4. **serde** - JSON serialization for CLI/bridge
5. **chrono** - Timestamp handling
6. **uuid** - Asset ID generation

## Build Command

```bash
cd forgeimages-core
cargo build
cargo test
```
