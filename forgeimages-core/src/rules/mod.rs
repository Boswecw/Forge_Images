//! Validation Rules
//!
//! Each rule category lives in its own submodule.
//! The existing 3 rules (aspect_ratio, resolution, color_count) are preserved
//! and re-exported here. New rules for Cover Forge are added alongside them.

pub mod aspect_ratio;
pub mod resolution;
pub mod color_count;
pub mod dimensions;
pub mod color_space;
pub mod file_size;

pub use aspect_ratio::AspectRatioRule;
pub use resolution::ResolutionRule as ResolutionCheckRule;
pub use color_count::ColorCountRule as ColorCountCheckRule;
pub use dimensions::DimensionMatchRule;
pub use color_space::ColorSpaceRule;
pub use file_size::FileSizeRule;
