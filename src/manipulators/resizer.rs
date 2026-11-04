use super::Manipulator;
use anyhow::{Context, Result};
use fast_image_resize::Resizer as FIRResizer;
use fast_image_resize::{IntoImageView, PixelType, images::Image};
use image::{DynamicImage, ImageBuffer};

#[derive(Debug, Clone, Copy)]
pub enum ResizeKind {
    Percentage(f32),
    Custom(u32, u32),
}

#[derive(Debug, Clone, Copy)]
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
        let pixel_type = image
            .pixel_type()
            .context("Unsupported pixel type for resizing")?;
        let mut dst_image = Image::new(width, height, pixel_type);

        let mut resizer = FIRResizer::new();
        resizer
            .resize(&image, &mut dst_image, None)
            .context("Failed to resize image")?;

        let raw_buffer = dst_image.into_vec();
        let resized = match pixel_type {
            PixelType::U8x4 => ImageBuffer::from_raw(width, height, raw_buffer).map(DynamicImage::ImageRgba8),
            PixelType::U8x3 => ImageBuffer::from_raw(width, height, raw_buffer).map(DynamicImage::ImageRgb8),
            PixelType::U8 => ImageBuffer::from_raw(width, height, raw_buffer).map(DynamicImage::ImageLuma8),
            // Handle other variants (U16, F32, etc.) as needed
            _ => return Err(anyhow::anyhow!("Unsupported pixel type")),
        };
        resized.context("Failed to create image buffer")
    }
}
