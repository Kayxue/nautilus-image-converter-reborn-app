use image::DynamicImage;
use std::io::Error;

pub trait Reader {
    fn read_images(&self, path: Vec<String>) -> Result<Vec<DynamicImage>, Error> {
        todo!()
    }

    fn write_images(&self, images: Vec<DynamicImage>) -> Result<(), Error> {
        todo!()
    }
}

pub trait Manipulator {
    fn manipulate_images(&self) -> Result<(), Error>;
}

pub struct ImageManipulator<T: Reader + Manipulator>(T);
