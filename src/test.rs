use std::path::{Path, PathBuf};

use crate::{
    OutputMode,
    manipulators::{
        resizer::{ResizeKind, Resizer, ResizerConfig},
        rotator::{RotationAngle, RotationAngleKind, Rotator, RotatorConfig},
    },
    window::{destination_path, process_single_image},
};

#[test]
fn test_destination_path_new_file() {
    let path = Path::new("/home/user/photo.jpg");
    let mode = OutputMode::NewFile(".rotated".to_string());
    let dest = destination_path(path, &mode);
    assert_eq!(dest, PathBuf::from("/home/user/photo.rotated.jpg"));
}

#[test]
fn test_destination_path_in_place() {
    let path = Path::new("/home/user/photo.png");
    let mode = OutputMode::InPlace;
    let dest = destination_path(path, &mode);
    assert_eq!(dest, PathBuf::from("/home/user/photo.png"));
}

#[test]
fn test_destination_path_no_extension() {
    let path = Path::new("/home/user/my_image");
    let mode = OutputMode::NewFile(".resized".to_string());
    let dest = destination_path(path, &mode);
    assert_eq!(dest, PathBuf::from("/home/user/my_image.resized"));
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
