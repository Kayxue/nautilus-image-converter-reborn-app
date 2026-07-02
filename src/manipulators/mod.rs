use anyhow::{Context, Result};
use image::{DynamicImage, ImageReader};

mod resizer;
mod rotator;

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
    fn manipulate_next_image(&self, image: DynamicImage) -> Result<()>;
}

pub struct ImageManipulator<T: Manipulator>(pub T);

impl<T: Manipulator> Reader for ImageManipulator<T> {}

impl<T: Manipulator> ImageManipulator<T> {
    pub fn new(manipulator: T) -> Self {
        Self(manipulator)
    }

    pub fn manipulate_image(&self, image: DynamicImage) -> Result<()> {
        self.0.manipulate_next_image(image)
    }
}
