use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use image::{DynamicImage, ImageFormat};

use crate::manipulators::Manipulator;

#[derive(Debug, Clone, Default)]
pub struct ConverterConfig {
    pub targets: HashMap<PathBuf, ImageFormat>,
    pub default: Option<ImageFormat>,
}

pub struct Converter(pub ConverterConfig);

impl Manipulator for Converter {
    fn manipulate_next_image(&self, image: DynamicImage) -> Result<DynamicImage> {
        Ok(image)
    }

    fn target_extension(&self, source: &Path) -> Option<String> {
        self.0
            .targets
            .get(source)
            .or(self.0.default.as_ref())
            .and_then(|f| f.extensions_str().first())
            .map(|ext| (*ext).to_owned())
    }
}
