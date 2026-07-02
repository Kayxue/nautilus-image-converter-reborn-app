use super::Manipulator;
use anyhow::Result;
use image::DynamicImage;

pub struct RotatorConfig {}

pub struct Rotator(pub RotatorConfig);

impl Manipulator for Rotator {
    fn manipulate_next_image(&self, image: &DynamicImage) -> Result<()> {
        todo!()
    }
}
