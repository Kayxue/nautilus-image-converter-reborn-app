use super::Manipulator;
use anyhow::{Context, Result};
use fast_image_resize::Resizer as FIRResizer;
use fast_image_resize::{IntoImageView, images::Image};
use image::{DynamicImage, ImageBuffer};

#[derive(Debug)]
pub enum ResizeKind {
    Percentage(f32),
    Custom(u32, u32),
}

pub struct ResizerConfig(pub ResizeKind);

pub struct Resizer(pub ResizerConfig);

impl Manipulator for Resizer {
    fn manipulate_next_image(&self, image: DynamicImage) -> Result<DynamicImage> {
        let ResizerConfig(kind) = &self.0;
        let (width, height) = match kind {
            ResizeKind::Percentage(percent) => {
                let width = (image.width() as f32 * *percent) as u32;
                let height = (image.height() as f32 * *percent) as u32;
                (width, height)
            }
            ResizeKind::Custom(w, h) => (*w, *h),
        };
        let mut dst_image = Image::new(width, height, image.pixel_type().unwrap());

        let mut resizer = FIRResizer::new();
        resizer.resize(&image, &mut dst_image, None).unwrap();

        let pixel_type = dst_image.pixel_type();
        let raw_buffer = dst_image.into_vec();

        match pixel_type {
            fast_image_resize::PixelType::U8x4 => {
                let buffer = ImageBuffer::from_raw(width, height, raw_buffer)
                    .with_context(|| "Failed to create RGBA8 image buffer")?;
                Ok(DynamicImage::ImageRgba8(buffer))
            }
            fast_image_resize::PixelType::U8x3 => {
                let buffer = ImageBuffer::from_raw(width, height, raw_buffer)
                    .with_context(|| "Failed to create RGB8 image buffer")?;
                Ok(DynamicImage::ImageRgb8(buffer))
            }
            fast_image_resize::PixelType::U8 => {
                let buffer = ImageBuffer::from_raw(width, height, raw_buffer)
                    .with_context(|| "Failed to create Luma8 image buffer")?;
                Ok(DynamicImage::ImageLuma8(buffer))
            }
            // Handle other variants (U16, F32, etc.) as needed
            _ => Err(anyhow::anyhow!("Unsupported pixel type")),
        }
    }
}
