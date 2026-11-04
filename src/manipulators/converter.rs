use std::path::Path;

use anyhow::Result;
use image::{DynamicImage, ImageFormat};

use crate::manipulators::Manipulator;

#[derive(Debug, Clone)]
pub struct ConverterConfig(pub ImageFormat);

/// Converts images to another format. The pixel data is left untouched; the
/// actual re-encoding happens when the image is saved with the new extension.
pub struct Converter(pub ConverterConfig);

impl Manipulator for Converter {
    fn manipulate_next_image(&self, image: DynamicImage) -> Result<DynamicImage> {
        Ok(image)
    }

    fn target_extension(&self, _source: &Path) -> Option<String> {
        self.0.0.extensions_str().first().map(|e| (*e).to_owned())
    }
}
