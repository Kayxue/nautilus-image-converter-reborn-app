use std::path::PathBuf;

use anyhow::{Context, Result};
use image::{DynamicImage, ImageReader};

pub mod resizer;
pub mod rotator;

pub trait Reader {
    fn read_image(&self, path: PathBuf) -> Result<DynamicImage> {
        ImageReader::open(path)?
            .with_guessed_format()?
            .decode()
            .with_context(|| "Failed to read image")
    }
}

pub trait Manipulator {
    fn manipulate_next_image(&self, image: DynamicImage) -> Result<DynamicImage>;
}

pub struct ImageManipulator<T: Manipulator>(pub T);

impl<T: Manipulator> Reader for ImageManipulator<T> {}

impl<T: Manipulator> ImageManipulator<T> {
    pub fn new(manipulator: T) -> Self {
        Self(manipulator)
    }

    pub fn manipulate_image(&self, image: DynamicImage) -> Result<DynamicImage> {
        self.0.manipulate_next_image(image)
    }
}
