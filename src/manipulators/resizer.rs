use super::Manipulator;
use anyhow::Result;
use image::DynamicImage;

pub struct ResizerConfig {}

pub struct Resizer(pub ResizerConfig);

impl Manipulator for Resizer {
    fn manipulate_next_image(&self, image: DynamicImage) -> Result<()> {
        todo!()
    }
}
