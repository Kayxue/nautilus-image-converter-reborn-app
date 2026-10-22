use std::path::Path;

use anyhow::Result;
use image::DynamicImage;

pub mod converter;
pub mod resizer;
pub mod rotator;

pub trait Manipulator: Send + Sync {
    fn manipulate_next_image(&self, image: DynamicImage) -> Result<DynamicImage>;

    /// Target file extension for `source`, or `None` to keep the original.
    fn target_extension(&self, _source: &Path) -> Option<String> {
        None
    }
}
