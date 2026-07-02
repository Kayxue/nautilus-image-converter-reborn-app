use anyhow::{Context, Result};
use image::{DynamicImage, ImageReader};

pub trait Reader {
    fn read_image(&self, path: String) -> Result<DynamicImage> {
        ImageReader::open(path)?
            .with_guessed_format()?
            .decode()
            .with_context(|| "Failed to read image")
    }

    fn write_image(&self, image: DynamicImage, path: String) -> Result<()> {
        image.save(path).with_context(|| "Failed to write image")
    }
}

pub trait Manipulator {
    fn manipulate_next_image(&self) -> Result<()>;
}

pub struct ImageManipulator<T: Reader + Manipulator>(T);
