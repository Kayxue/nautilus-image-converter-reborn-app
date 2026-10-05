use argh::{FromArgValue, FromArgs};
use relm4::RelmApp;
use std::path::PathBuf;
use strum_macros::Display;

use crate::window::{AppModel, Initializer};

mod manipulators;
mod window;
mod x11;

#[cfg(test)]
mod test;

#[derive(FromArgValue, Debug, Display, Clone)]
#[strum(serialize_all = "title_case")]
pub enum Mode {
    Resize,
    Rotate,
    Convert,
}

#[derive(Debug, Clone)]
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
    /// XID of the parent (Nautilus) window to set as transient-for (X11 only).
    #[argh(option)]
    parent_xid: Option<u64>,
    /// exported handle token of the parent (Nautilus) window (Wayland XDG Foreign).
    #[argh(option)]
    parent_handle: Option<String>,
}

fn main() {
    let Args {
        mode,
        paths,
        parent_xid,
        parent_handle,
    } = argh::from_env();

    if paths.is_empty() {
        eprintln!("Error: at least one input path is required.");
        std::process::exit(1);
    }

    let paths_buf: Vec<PathBuf> = paths.iter().map(|e| e.parse().unwrap()).collect();

    if paths_buf.iter().any(|e| !e.exists()) {
        eprintln!("Error: one or more input paths do not exist.");
        std::process::exit(1);
    }

    let initializer = Initializer {
        mode,
        paths,
        parent_xid,
        parent_handle,
    };

    let relm = RelmApp::new("com.kay.nautilus_image_converter").with_args(vec![]);
    relm.run::<AppModel>(initializer);
}
