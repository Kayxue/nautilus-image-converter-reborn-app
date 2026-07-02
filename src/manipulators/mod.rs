use anyhow::{Context, Result};
use image::{DynamicImage, ImageReader};

pub trait Reader {
    fn read_image(&self, path: String) -> Result<DynamicImage> {
        ImageReader::open(path)?
            .decode()
            .with_context(|| "Failed to read image")
    }

    fn write_image(&self, image: DynamicImage, path: String) -> Result<()> {
        image.save(path).with_context(|| "Failed to write image")
    }
}

pub trait Manipulator {
    fn manipulate_images(&self) -> Result<ProcessedResult>;
}

pub struct ImageManipulator<T: Manipulator>(T);

pub struct ProcessedResult {
    pub total: u32,
    pub success: u32,
    pub errors: u32,
}
