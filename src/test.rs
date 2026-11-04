use std::path::{Path, PathBuf};

use image::ImageFormat;

use crate::{
    manipulators::{
        converter::{Converter, ConverterConfig},
        resizer::{ResizeKind, Resizer, ResizerConfig},
        rotator::{RotationAngle, RotationAngleKind, Rotator, RotatorConfig},
    },
    output::OutputMode,
    window::process_single_image,
};

#[test]
fn test_destination_path_new_file() {
    let path = Path::new("/home/user/photo.jpg");
    let mode = OutputMode::NewFile(".rotated".to_string());
    let dest = mode.destination(path, None);
    assert_eq!(dest, PathBuf::from("/home/user/photo.rotated.jpg"));
}

#[test]
fn test_destination_path_in_place() {
    let path = Path::new("/home/user/photo.png");
    let mode = OutputMode::InPlace;
    let dest = mode.destination(path, None);
    assert_eq!(dest, PathBuf::from("/home/user/photo.png"));
    assert!(!mode.removes_original(path, &dest));
}

#[test]
fn test_destination_path_no_extension() {
    let path = Path::new("/home/user/my_image");
    let mode = OutputMode::NewFile(".resized".to_string());
    let dest = mode.destination(path, None);
    assert_eq!(dest, PathBuf::from("/home/user/my_image.resized"));
}

#[test]
fn test_destination_path_convert() {
    let path = Path::new("/home/user/photo.jpg");

    let keep = OutputMode::NewFile(String::new());
    let dest = keep.destination(path, Some("png"));
    assert_eq!(dest, PathBuf::from("/home/user/photo.png"));
    assert!(!keep.removes_original(path, &dest));

    let replace = OutputMode::InPlace;
    let dest = replace.destination(path, Some("png"));
    assert_eq!(dest, PathBuf::from("/home/user/photo.png"));
    assert!(replace.removes_original(path, &dest));
}

#[test]
fn test_process_single_image_rotate() {
    let temp_dir = std::env::temp_dir();
    let src_path = temp_dir.join("test_rotate_src.png");
    let dest_path = temp_dir.join("test_rotate_src.rotated.png");
    let _ = std::fs::remove_file(&src_path);
    let _ = std::fs::remove_file(&dest_path);

    let img = image::DynamicImage::ImageRgba8(image::ImageBuffer::new(10, 20));
    img.save(&src_path).unwrap();

    let rotator = Rotator(RotatorConfig(RotationAngle::Specific(
        RotationAngleKind::Ninety,
    )));
    let output_mode = OutputMode::NewFile(".rotated".to_string());

    process_single_image(&src_path, &output_mode, &rotator).unwrap();

    assert!(dest_path.exists());
    let rotated_img = image::ImageReader::open(&dest_path)
        .unwrap()
        .decode()
        .unwrap();
    assert_eq!(rotated_img.width(), 20);
    assert_eq!(rotated_img.height(), 10);

    let _ = std::fs::remove_file(&src_path);
    let _ = std::fs::remove_file(&dest_path);
}

#[test]
fn test_process_single_image_resize() {
    let temp_dir = std::env::temp_dir();
    let src_path = temp_dir.join("test_resize_src.png");
    let dest_path = temp_dir.join("test_resize_src.resized.png");
    let _ = std::fs::remove_file(&src_path);
    let _ = std::fs::remove_file(&dest_path);

    let img = image::DynamicImage::ImageRgba8(image::ImageBuffer::new(100, 100));
    img.save(&src_path).unwrap();

    let resizer = Resizer(ResizerConfig(ResizeKind::Percentage(0.5)));
    let output_mode = OutputMode::NewFile(".resized".to_string());

    process_single_image(&src_path, &output_mode, &resizer).unwrap();

    assert!(dest_path.exists());
    let resized_img = image::ImageReader::open(&dest_path)
        .unwrap()
        .decode()
        .unwrap();
    assert_eq!(resized_img.width(), 50);
    assert_eq!(resized_img.height(), 50);

    let _ = std::fs::remove_file(&src_path);
    let _ = std::fs::remove_file(&dest_path);
}

#[test]
fn test_process_single_image_convert_keep_original() {
    let temp_dir = std::env::temp_dir();
    let src_path = temp_dir.join("test_convert_keep_src.png");
    let dest_path = temp_dir.join("test_convert_keep_src.jpg");
    let _ = std::fs::remove_file(&src_path);
    let _ = std::fs::remove_file(&dest_path);

    // RGBA source to make sure alpha is dropped for JPEG.
    let img = image::DynamicImage::ImageRgba8(image::ImageBuffer::new(8, 6));
    img.save(&src_path).unwrap();

    let converter = Converter(ConverterConfig(ImageFormat::Jpeg));
    process_single_image(&src_path, &OutputMode::NewFile(String::new()), &converter).unwrap();

    assert!(src_path.exists());
    let reader = image::ImageReader::open(&dest_path)
        .unwrap()
        .with_guessed_format()
        .unwrap();
    assert_eq!(reader.format(), Some(ImageFormat::Jpeg));
    let converted = reader.decode().unwrap();
    assert_eq!((converted.width(), converted.height()), (8, 6));

    let _ = std::fs::remove_file(&src_path);
    let _ = std::fs::remove_file(&dest_path);
}

#[test]
fn test_process_single_image_convert_remove_original() {
    let temp_dir = std::env::temp_dir();
    let src_path = temp_dir.join("test_convert_remove_src.png");
    let dest_path = temp_dir.join("test_convert_remove_src.bmp");
    let _ = std::fs::remove_file(&src_path);
    let _ = std::fs::remove_file(&dest_path);

    let img = image::DynamicImage::ImageRgb8(image::ImageBuffer::new(4, 4));
    img.save(&src_path).unwrap();

    let converter = Converter(ConverterConfig(ImageFormat::Bmp));
    process_single_image(&src_path, &OutputMode::InPlace, &converter).unwrap();

    assert!(!src_path.exists());
    assert!(dest_path.exists());

    let _ = std::fs::remove_file(&dest_path);
}

/// Every format offered in the convert dropdown must be writable from a
/// typical RGBA source.
#[test]
fn test_convert_to_all_writable_formats() {
    let temp_dir = std::env::temp_dir();
    let src_path = temp_dir.join("test_convert_all_src.png");
    let img = image::DynamicImage::ImageRgba8(image::ImageBuffer::new(16, 16));
    img.save(&src_path).unwrap();

    let mut failures = Vec::new();
    for (format, ext) in crate::window::writable_formats() {
        let mode = OutputMode::NewFile(".all".to_owned());
        let converter = Converter(ConverterConfig(format));
        let dest = mode.destination(&src_path, Some(ext));
        if let Err(e) = process_single_image(&src_path, &mode, &converter) {
            failures.push(format!("{:?}: {:#}", format, e));
        }
        let _ = std::fs::remove_file(&dest);
    }
    let _ = std::fs::remove_file(&src_path);

    assert!(
        failures.is_empty(),
        "failed formats:\n{}",
        failures.join("\n")
    );
}
