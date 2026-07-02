use argh::{FromArgValue, FromArgs};
use std::path::PathBuf;

mod manipulators;
mod window;

#[derive(FromArgValue, Debug)]
pub enum Mode {
    Resize,
    Rotate,
    Convert,
}

#[derive(FromArgs)]
/// A image tool that can resize, rotate, and convert images.
struct Args {
    /// operation mode: resize, rotate, or convert.
    #[argh(option, short = 'm')]
    mode: Option<Mode>,
    /// path to the input image file.
    #[argh(option, short = 'p')]
    path: Option<String>,
    /// path to the output image file.
    #[argh(option, short = 'o')]
    output: Option<PathBuf>,
}

fn main() {
    let Args { mode, path, output } = argh::from_env();
    if mode.is_none() || path.is_none() || output.is_none() {
        eprintln!("Error: mode, path, and output are required.");
        std::process::exit(1);
    }

    let paths: Vec<PathBuf> = path
        .unwrap()
        .split(',')
        .map(|e| e.parse().unwrap())
        .collect::<Vec<_>>();

    if paths.iter().any(|e| !e.exists()) {
        eprintln!("Error: one or more input paths do not exist.");
        std::process::exit(1);
    }

    if !output.as_ref().unwrap().exists() {
        eprintln!("Error: output path does not exist.");
        std::process::exit(1);
    }

    if !output.as_ref().unwrap().is_dir() {
        eprintln!("Error: output path is not a directory.");
        std::process::exit(1);
    }

    println!("Hello, world!");
}
