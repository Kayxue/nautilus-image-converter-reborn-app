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

pub struct ImageManipulator(pub Box<dyn Manipulator>);

impl Reader for ImageManipulator {}

impl ImageManipulator {
    pub fn new(manipulator: Box<dyn Manipulator>) -> Self {
        Self(manipulator)
    }

    pub fn manipulate_image(&self, image: DynamicImage) -> Result<DynamicImage> {
        self.0.manipulate_next_image(image)
    }
}
