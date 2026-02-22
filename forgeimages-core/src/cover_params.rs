//! Dynamic Template Resolution (Cover Parameters)
//!
//! Resolves parameterized templates into concrete validation rules.
//! Static templates (like pwa-icon) pass through unchanged.
//! Dynamic templates (like book-cover) compute dimensions from page count + paper stock.
//!
//! This is a new capability layer on top of the existing TemplateRegistry.
//! The registry loads static templates; this module resolves them into concrete rules.

use serde::{Deserialize, Serialize};

/// Parameters for dynamically computing book cover dimensions.
///
/// Cover dimensions depend on trim size, page count, and paper stock.
/// The spine width is calculated from page count × PPI (pages per inch).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverTemplateParams {
    /// Trim width in inches (e.g., 6.0 for 6×9 book)
    pub trim_width_in: f64,
    /// Trim height in inches (e.g., 9.0 for 6×9 book)
    pub trim_height_in: f64,
    /// Total page count (used for spine calculation)
    pub page_count: u32,
    /// Paper stock type
    pub paper_stock: PaperStock,
    /// Bleed in inches (default 0.125)
    #[serde(default = "default_bleed")]
    pub bleed_in: f64,
    /// Target DPI for print (default 300)
    #[serde(default = "default_dpi")]
    pub dpi: u32,
}

fn default_bleed() -> f64 {
    0.125
}
fn default_dpi() -> u32 {
    300
}

/// Paper stock determines pages-per-inch (PPI) for spine width calculation.
///
/// Formulas from KDP and IngramSpark specifications.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaperStock {
    /// KDP White: pages × 0.002252 + 0.06"
    KdpWhite,
    /// KDP Cream: pages × 0.0025 + 0.06"
    KdpCream,
    /// IngramSpark White 50#: pages / 444 PPI
    IngramWhite50,
    /// IngramSpark Crème 50#: pages / 400 PPI
    IngramCreme50,
    /// IngramSpark Color 80#: pages / 340 PPI
    IngramColor80,
}

/// Resolved cover dimensions in both inches and pixels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedCoverDimensions {
    /// Spine width in inches
    pub spine_width_in: f64,
    /// Total cover width in inches (front + spine + back + 2× bleed)
    pub total_width_in: f64,
    /// Total cover height in inches (trim + 2× bleed)
    pub total_height_in: f64,
    /// Total cover width in pixels at target DPI
    pub total_width_px: u32,
    /// Total cover height in pixels at target DPI
    pub total_height_px: u32,
    /// Bleed in inches
    pub bleed_in: f64,
    /// DPI used for calculation
    pub dpi: u32,
}

impl CoverTemplateParams {
    /// Calculate spine width based on paper stock and page count.
    pub fn spine_width(&self) -> f64 {
        match self.paper_stock {
            PaperStock::KdpWhite => self.page_count as f64 * 0.002252 + 0.06,
            PaperStock::KdpCream => self.page_count as f64 * 0.0025 + 0.06,
            PaperStock::IngramWhite50 => self.page_count as f64 / 444.0,
            PaperStock::IngramCreme50 => self.page_count as f64 / 400.0,
            PaperStock::IngramColor80 => self.page_count as f64 / 340.0,
        }
    }

    /// Resolve to concrete pixel dimensions.
    pub fn resolve(&self) -> ResolvedCoverDimensions {
        let spine = self.spine_width();
        // Total width = back cover + spine + front cover + bleed on both sides
        let total_width_in = (self.trim_width_in * 2.0) + spine + (self.bleed_in * 2.0);
        let total_height_in = self.trim_height_in + (self.bleed_in * 2.0);

        let total_width_px = (total_width_in * self.dpi as f64).round() as u32;
        let total_height_px = (total_height_in * self.dpi as f64).round() as u32;

        ResolvedCoverDimensions {
            spine_width_in: spine,
            total_width_in,
            total_height_in,
            total_width_px,
            total_height_px,
            bleed_in: self.bleed_in,
            dpi: self.dpi,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdp_white_spine_300_pages() {
        let params = CoverTemplateParams {
            trim_width_in: 6.0,
            trim_height_in: 9.0,
            page_count: 300,
            paper_stock: PaperStock::KdpWhite,
            bleed_in: 0.125,
            dpi: 300,
        };
        let spine = params.spine_width();
        // 300 × 0.002252 + 0.06 = 0.7356
        assert!((spine - 0.7356).abs() < 0.001);
    }

    #[test]
    fn test_resolve_dimensions() {
        let params = CoverTemplateParams {
            trim_width_in: 6.0,
            trim_height_in: 9.0,
            page_count: 300,
            paper_stock: PaperStock::KdpWhite,
            bleed_in: 0.125,
            dpi: 300,
        };
        let dims = params.resolve();
        // Width = 6 + 6 + 0.7356 + 0.25 = 12.9856"
        // At 300 DPI = 3895.68 → 3896 px
        assert!((dims.total_width_in - 12.9856).abs() < 0.01);
        assert_eq!(dims.total_height_px, 2775); // 9.25" × 300
    }

    #[test]
    fn test_ingram_creme_spine() {
        let params = CoverTemplateParams {
            trim_width_in: 5.5,
            trim_height_in: 8.5,
            page_count: 200,
            paper_stock: PaperStock::IngramCreme50,
            bleed_in: 0.125,
            dpi: 300,
        };
        let spine = params.spine_width();
        // 200 / 400 = 0.5"
        assert!((spine - 0.5).abs() < 0.001);
    }
}
