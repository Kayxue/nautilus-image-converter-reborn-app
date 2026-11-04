//! Output handling: where a processed image is written, how it is encoded,
//! and what happens to the original file afterwards.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::{DynamicImage, ImageError, ImageFormat};

/// Describes where a processed image is written and what happens to the
/// original file.
///
/// | Mode             | Resize / Rotate                    | Convert                                   |
/// |------------------|------------------------------------|-------------------------------------------|
/// | `InPlace`        | overwrite `photo.jpg`              | write `photo.png`, remove `photo.jpg`     |
/// | `NewFile(sfx)`   | write `photo{sfx}.jpg`, keep orig. | write `photo{sfx}.png`, keep `photo.jpg`  |
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputMode {
    /// Replace the original file. When the extension changes (format
    /// conversion), the new file is written next to the original and the
    /// original is removed after a successful save.
    InPlace,
    /// Keep the original file and write a new one next to it, with the given
    /// suffix appended to the file stem (the suffix may be empty).
    NewFile(String),
}

impl OutputMode {
    /// Computes the destination path for `original`.
    ///
    /// `target_ext` replaces the file extension when given (format
    /// conversion); otherwise the original extension is kept.
    pub fn destination(&self, original: &Path, target_ext: Option<&str>) -> PathBuf {
        match self {
            OutputMode::InPlace => match target_ext {
                Some(ext) => original.with_extension(ext),
                None => original.to_path_buf(),
            },
            OutputMode::NewFile(suffix) => {
                let parent = original.parent().unwrap_or_else(|| Path::new(""));
                let file_stem = original
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("image");
                let extension =
                    target_ext.or_else(|| original.extension().and_then(|e| e.to_str()));
                let new_filename = match extension {
                    Some(ext) => format!("{}{}.{}", file_stem, suffix, ext),
                    None => format!("{}{}", file_stem, suffix),
                };
                parent.join(new_filename)
            }
        }
    }

    /// Whether the original file must be removed once `dest` was written.
    pub fn removes_original(&self, original: &Path, dest: &Path) -> bool {
        matches!(self, OutputMode::InPlace) && original != dest
    }
}

/// Writes `img` to `dest`, choosing the encoder from the file extension.
///
/// The image is saved as-is when possible, so color type and bit depth are
/// preserved. If the encoder rejects the color type (e.g. RGBA → JPEG), the
/// pixel data is converted to a compatible color type and saved again.
pub fn save_image(img: &DynamicImage, dest: &Path) -> Result<()> {
    let format = ImageFormat::from_path(dest)
        .with_context(|| format!("Unsupported output format: {:?}", dest))?;

    match img.save_with_format(dest, format) {
        Err(ImageError::Unsupported(_)) => to_compatible_color(img, format)
            .save_with_format(dest, format)
            .with_context(|| format!("Failed to save image to {:?}", dest)),
        other => other.with_context(|| format!("Failed to save image to {:?}", dest)),
    }
}

/// Converts `img` into a color type accepted by the encoder for `format`.
fn to_compatible_color(img: &DynamicImage, format: ImageFormat) -> DynamicImage {
    let has_alpha = img.color().has_alpha();
    match format {
        // 8-bit, no alpha channel.
        ImageFormat::Jpeg | ImageFormat::Pnm => DynamicImage::ImageRgb8(img.to_rgb8()),
        // 16-bit RGBA only.
        ImageFormat::Farbfeld => DynamicImage::ImageRgba16(img.to_rgba16()),
        // Floating point formats.
        ImageFormat::Hdr => DynamicImage::ImageRgb32F(img.to_rgb32f()),
        ImageFormat::OpenExr if has_alpha => DynamicImage::ImageRgba32F(img.to_rgba32f()),
        ImageFormat::OpenExr => DynamicImage::ImageRgb32F(img.to_rgb32f()),
        // Everything else accepts 8-bit RGB(A).
        _ if has_alpha => DynamicImage::ImageRgba8(img.to_rgba8()),
        _ => DynamicImage::ImageRgb8(img.to_rgb8()),
    }
}
