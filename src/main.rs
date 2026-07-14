use argh::{FromArgValue, FromArgs};
use relm4::RelmApp;
use std::path::PathBuf;
use strum_macros::Display;

use crate::window::{AppModel, Initializer};

mod manipulators;
mod window;

#[derive(FromArgValue, Debug, Display, Clone)]
#[strum(serialize_all = "title_case")]
pub enum Mode {
    Resize,
    Rotate,
    Convert,
}

#[derive(Debug)]
pub enum OutputMode {
    InPlace,
    NewFile(String),
}

#[derive(FromArgs)]
/// A image tool that can resize, rotate, and convert images.
struct Args {
    /// operation mode: resize, rotate, or convert.
    #[argh(positional)]
    mode: Mode,
    /// paths to the input image files.
    #[argh(positional)]
    paths: Vec<String>,
}

fn main() {
    let Args { mode, paths } = argh::from_env();

    if paths.is_empty() {
        eprintln!("Error: at least one input path is required.");
        std::process::exit(1);
    }

    let paths_buf: Vec<PathBuf> = paths.iter().map(|e| e.parse().unwrap()).collect();

    if paths_buf.iter().any(|e| !e.exists()) {
        eprintln!("Error: one or more input paths do not exist.");
        std::process::exit(1);
    }

    let initializer = Initializer { mode, paths };

    let relm = RelmApp::new("com.kay.nautilus_image_converter")
        .with_args(vec![]);
    relm.run::<AppModel>(initializer);
}
