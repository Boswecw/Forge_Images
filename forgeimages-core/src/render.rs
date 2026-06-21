//! Deterministic SVG → raster rendering.
//!
//! This module implements Law 1 (SVG Is Truth) and Law 4 (Deterministic Output):
//! a vector master is rasterized to each export format/size. The same SVG bytes
//! and the same target size always produce identical output bytes.
//!
//! AI/generative production lives OUTSIDE this engine (NeuroForge). The engine
//! only rasterizes a provided or template-derived vector master — it never
//! invents imagery.

use std::io::Cursor;

use thiserror::Error;

use crate::templates::ExportFormat;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Invalid SVG source: {0}")]
    InvalidSvg(String),

    #[error("Target size {0}x{1} is invalid")]
    InvalidSize(u32, u32),

    #[error("Rasterization failed: {0}")]
    Rasterize(String),

    #[error("Encoding to {0:?} failed: {1}")]
    Encode(ExportFormat, String),

    #[error("Format {0:?} is not supported for rendering")]
    UnsupportedFormat(ExportFormat),
}

/// Build a deterministic default vector master sized to `[w, h]`.
///
/// Used when a caller compiles without supplying `source_data`. The output is a
/// pure function of the dimensions, preserving Law 4 (deterministic output).
pub fn default_master_svg(width: u32, height: u32) -> String {
    // A neutral, content-free placeholder master: a rounded panel with a
    // centered ring. Deterministic — depends only on width/height.
    let r = (width.min(height) as f32) * 0.30;
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let radius = (width.min(height) as f32) * 0.12;
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" \
viewBox=\"0 0 {w} {h}\">\
<rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" rx=\"{rx}\" fill=\"#1b1f2a\"/>\
<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{ring}\" fill=\"none\" stroke=\"#c9a227\" \
stroke-width=\"{sw}\"/></svg>",
        w = width,
        h = height,
        rx = radius,
        cx = cx,
        cy = cy,
        ring = r,
        sw = (width.min(height) as f32) * 0.04,
    )
}

/// Render an SVG master to the requested format at `[width, height]`.
///
/// For `Svg`, the master is normalized to the target viewBox and returned as-is
/// (vector stays vector). For raster formats the SVG is rasterized via resvg and
/// encoded deterministically.
pub fn render(
    svg_source: &str,
    format: &ExportFormat,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, RenderError> {
    if width == 0 || height == 0 {
        return Err(RenderError::InvalidSize(width, height));
    }

    match format {
        ExportFormat::Svg => Ok(svg_source.as_bytes().to_vec()),
        ExportFormat::Png => encode_raster(svg_source, width, height, ExportFormat::Png),
        ExportFormat::Jpg => encode_raster(svg_source, width, height, ExportFormat::Jpg),
        ExportFormat::Ico => encode_raster(svg_source, width, height, ExportFormat::Ico),
        ExportFormat::Pdf => Err(RenderError::UnsupportedFormat(ExportFormat::Pdf)),
    }
}

/// Rasterize the SVG to un-premultiplied RGBA, then encode to the target format.
fn encode_raster(
    svg_source: &str,
    width: u32,
    height: u32,
    format: ExportFormat,
) -> Result<Vec<u8>, RenderError> {
    let rgba = rasterize_rgba(svg_source, width, height)?;

    // PNG: tiny_skia already produced clean un-premultiplied RGBA; route through
    // the `image` crate for a single consistent encoding path across formats.
    let img = image::RgbaImage::from_raw(width, height, rgba)
        .ok_or_else(|| RenderError::Rasterize("RGBA buffer size mismatch".into()))?;

    let mut buf = Cursor::new(Vec::new());
    match format {
        ExportFormat::Png => image::DynamicImage::ImageRgba8(img)
            .write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| RenderError::Encode(format.clone(), e.to_string()))?,
        ExportFormat::Ico => image::DynamicImage::ImageRgba8(img)
            .write_to(&mut buf, image::ImageFormat::Ico)
            .map_err(|e| RenderError::Encode(format.clone(), e.to_string()))?,
        // JPEG has no alpha channel — flatten to RGB first.
        ExportFormat::Jpg => image::DynamicImage::ImageRgba8(img)
            .to_rgb8()
            .write_to(&mut buf, image::ImageFormat::Jpeg)
            .map_err(|e| RenderError::Encode(format.clone(), e.to_string()))?,
        other => return Err(RenderError::UnsupportedFormat(other)),
    }
    Ok(buf.into_inner())
}

/// Rasterize the SVG to an un-premultiplied RGBA byte buffer at `[width, height]`.
fn rasterize_rgba(svg_source: &str, width: u32, height: u32) -> Result<Vec<u8>, RenderError> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg_source, &opt)
        .map_err(|e| RenderError::InvalidSvg(e.to_string()))?;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or(RenderError::InvalidSize(width, height))?;

    let size = tree.size();
    let scale_x = width as f32 / size.width();
    let scale_y = height as f32 / size.height();
    let transform = resvg::tiny_skia::Transform::from_scale(scale_x, scale_y);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // tiny_skia stores premultiplied alpha; demultiply for a true RGBA buffer.
    let mut rgba = Vec::with_capacity((width as usize) * (height as usize) * 4);
    for px in pixmap.pixels() {
        let c = px.demultiply();
        rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Ok(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_master_is_deterministic() {
        assert_eq!(default_master_svg(512, 512), default_master_svg(512, 512));
    }

    #[test]
    fn png_render_is_deterministic_and_nonempty() {
        let svg = default_master_svg(64, 64);
        let a = render(&svg, &ExportFormat::Png, 64, 64).unwrap();
        let b = render(&svg, &ExportFormat::Png, 64, 64).unwrap();
        assert_eq!(a, b, "same SVG + size must yield identical PNG bytes");
        assert!(a.len() > 67, "expected a real PNG, not a 1x1 stub");
        assert_eq!(&a[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[test]
    fn ico_and_jpg_render() {
        let svg = default_master_svg(32, 32);
        assert!(!render(&svg, &ExportFormat::Ico, 32, 32).unwrap().is_empty());
        assert!(!render(&svg, &ExportFormat::Jpg, 32, 32).unwrap().is_empty());
    }

    #[test]
    fn svg_passthrough_returns_vector() {
        let svg = default_master_svg(16, 16);
        let out = render(&svg, &ExportFormat::Svg, 16, 16).unwrap();
        assert_eq!(out, svg.as_bytes());
    }

    #[test]
    fn pdf_is_unsupported() {
        let svg = default_master_svg(16, 16);
        assert!(matches!(
            render(&svg, &ExportFormat::Pdf, 16, 16),
            Err(RenderError::UnsupportedFormat(ExportFormat::Pdf))
        ));
    }

    #[test]
    fn zero_size_rejected() {
        let svg = default_master_svg(16, 16);
        assert!(matches!(
            render(&svg, &ExportFormat::Png, 0, 16),
            Err(RenderError::InvalidSize(0, 16))
        ));
    }
}
