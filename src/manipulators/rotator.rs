use std::f32::consts::PI;

use super::Manipulator;
use anyhow::{Ok, Result};
use image::{DynamicImage, Rgba, imageops};
use imageproc::geometric_transformations::{
    Border::Constant, Interpolation, rotate_about_center_no_crop,
};

#[derive(Debug, Clone, Copy)]
pub enum RotationAngleKind {
    Ninety,
    HundredEighty,
    TwoHundredSeventy,
}

#[derive(Debug, Clone, Copy)]
pub enum RotationAngle {
    Specific(RotationAngleKind),
    Custom(u32),
}

#[derive(Debug, Clone, Copy)]
pub struct RotatorConfig(pub RotationAngle);

pub struct Rotator(pub RotatorConfig);

impl Manipulator for Rotator {
    fn manipulate_next_image(&self, image: DynamicImage) -> Result<DynamicImage> {
        let RotatorConfig(angle) = &self.0;
        match angle {
            RotationAngle::Specific(kind) => self.rotate_image_specific(image, kind),
            // Multiples of 90° use the lossless fast path.
            RotationAngle::Custom(deg) => match deg % 360 {
                0 => Ok(image),
                90 => self.rotate_image_specific(image, &RotationAngleKind::Ninety),
                180 => self.rotate_image_specific(image, &RotationAngleKind::HundredEighty),
                270 => self.rotate_image_specific(image, &RotationAngleKind::TwoHundredSeventy),
                deg => self.rotate_image_custom(image, deg),
            },
        }
    }
}

impl Rotator {
    fn rotate_image_specific(
        &self,
        image: DynamicImage,
        kind: &RotationAngleKind,
    ) -> Result<DynamicImage> {
        match kind {
            RotationAngleKind::Ninety => Ok(imageops::rotate90(&image).into()),
            RotationAngleKind::HundredEighty => Ok(imageops::rotate180(&image).into()),
            RotationAngleKind::TwoHundredSeventy => Ok(imageops::rotate270(&image).into()),
        }
    }

    fn rotate_image_custom(&self, image: DynamicImage, deg: u32) -> Result<DynamicImage> {
        Ok(rotate_about_center_no_crop(
            &image.to_rgba8(),
            (deg as f32 * PI) / 180.0,
            Interpolation::Bilinear,
            Constant(Rgba([255, 255, 255, 255])),
        )
        .into())
    }
}
