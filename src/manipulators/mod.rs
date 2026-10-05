use anyhow::Result;
use image::DynamicImage;

pub mod resizer;
pub mod rotator;

pub trait Manipulator: Send + Sync {
    fn manipulate_next_image(&self, image: DynamicImage) -> Result<DynamicImage>;
}
