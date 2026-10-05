use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use anyhow::{Context, Result};
use gtk::{
    Align, Box, Button, HeaderBar, Label, Orientation, ProgressBar, Spinner, Stack,
    StackTransitionType, Window,
    gio::prelude::ApplicationExt,
    prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt},
};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, component, main_application,
};

use crate::{
    Mode, OutputMode,
    manipulators::{
        Manipulator,
        resizer::{ResizeKind, Resizer, ResizerConfig},
        rotator::{RotationAngle, RotationAngleKind, Rotator, RotatorConfig},
    },
    window::window_body::{ResizeBodyModel, ResizeBodyOutput, RotateBodyModel, RotateBodyOutput},
};

mod window_body;

pub struct Initializer {
    pub mode: Mode,
    pub paths: Vec<String>,
    /// XID of the parent (Nautilus) window passed by the extension (X11 only).
    pub parent_xid: Option<u64>,
    /// Exported handle token of the parent (Nautilus) window (Wayland XDG Foreign).
    pub parent_handle: Option<String>,
}

pub struct GeneralConfig {
    pub mode: Mode,
    pub rotation_angle: Option<RotationAngle>,
    pub image_size: Option<ResizeKind>,
    pub output_mode: OutputMode,
}

struct HeaderModel {
    mode: Mode,
}

#[derive(Debug)]
enum HeaderOutput {
    Cancel,
    Proceed,
}

#[component(pub)]
impl SimpleComponent for HeaderModel {
    type Init = Mode;
    type Input = ();
    type Output = HeaderOutput;

    view! {
        #[root]
        HeaderBar {
            set_show_title_buttons: false,
            #[wrap(Some)]
            set_title_widget = &Label {
                set_label: &format!("{} Images", model.mode),
                add_css_class: "title",
            },
            pack_start = &Button {
                set_label: "Cancel",
                connect_clicked[sender] => move |_|{
                    sender.output(HeaderOutput::Cancel).unwrap()
                },
            },
            pack_end = &Button {
                set_label: &format!("{}", model.mode),
                add_css_class: "suggested-action",
                connect_clicked[sender] => move |_|{
                    sender.output(HeaderOutput::Proceed).unwrap()
                },
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HeaderModel { mode: params };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}

/// Keeps the body component controller alive for the lifetime of the window.
/// If this is dropped the output channel closes and signal-handler `.unwrap()`s panic.
#[allow(dead_code)]
enum BodyController {
    Resize(Controller<ResizeBodyModel>),
    Rotate(Controller<RotateBodyModel>),
}

pub struct AppModel {
    general_config: GeneralConfig,
    header: Controller<HeaderModel>,
    paths: Vec<String>,
    is_processing: bool,
    processed_count: usize,
    total_count: usize,
    /// Must be kept alive — holds the Sender half of the body's output channel.
    _body: BodyController,
}

#[derive(Debug)]
pub enum AppInput {
    Cancel,
    Execute,
    UpdateImageSize(ResizeKind),
    UpdateAngle(RotationAngle),
    UpdateOutputMode(OutputMode),
    ProgressStep(usize),
    Finished,
}

#[component(pub)]
impl SimpleComponent for AppModel {
    type Init = Initializer;

    type Input = AppInput;
    type Output = ();

    view! {
        #[root]
        Window {
            set_title: Some(&format!("{} Images", model.general_config.mode)),
            set_titlebar: Some(model.header.widget()),
            set_resizable: false,
            set_default_width: 440,

            #[name(stack)]
            Stack {
                set_transition_type: StackTransitionType::Crossfade,

                if model.is_processing {
                    Box {
                        set_orientation: Orientation::Vertical,
                        set_spacing: 16,
                        set_margin_all: 24,
                        set_hexpand: true,
                        set_vexpand: true,
                        set_valign: Align::Center,

                        #[name(spinner)]
                        Spinner {
                            #[watch]
                            set_spinning: model.is_processing,
                            set_size_request: (40, 40),
                            set_halign: Align::Center,
                        },

                        Label {
                            set_label: "Processing......",
                            set_halign: Align::Center,
                            add_css_class: "title-3",
                        },

                        #[name(progress_bar)]
                        ProgressBar {
                            set_hexpand: true,
                            set_show_text: true,
                            #[watch]
                            set_fraction: if model.total_count > 0 {
                                model.processed_count as f64 / model.total_count as f64
                            } else {
                                0.0
                            },
                            #[watch]
                            set_text: Some(&format!("{}/{}", model.processed_count, model.total_count)),
                        },
                    }
                } else {
                    #[name(dialog_vbox1)]
                    Box {
                        set_orientation: Orientation::Vertical,
                        set_spacing: 6,
                        set_margin_all: 12,
                        set_hexpand: true,
                        set_vexpand: true,
                    }
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header: Controller<HeaderModel> = HeaderModel::builder()
            .launch(init.mode.clone())
            .forward(sender.input_sender(), |msg| match msg {
                HeaderOutput::Cancel => AppInput::Cancel,
                HeaderOutput::Proceed => AppInput::Execute,
            });

        let (body_widget, model) = match init.mode {
            Mode::Resize => {
                let resize_body: Controller<ResizeBodyModel> = ResizeBodyModel::builder()
                    .launch(())
                    .forward(sender.input_sender(), |msg| match msg {
                        ResizeBodyOutput::UpdateImageSize(kind) => AppInput::UpdateImageSize(kind),
                        ResizeBodyOutput::UpdateOutputMode(mode) => {
                            AppInput::UpdateOutputMode(mode)
                        }
                    });
                let widget = resize_body.widget().clone();
                let model = AppModel {
                    general_config: GeneralConfig {
                        mode: Mode::Resize,
                        rotation_angle: None,
                        image_size: Some(ResizeKind::Percentage(0.5)),
                        output_mode: OutputMode::NewFile(".resized".to_owned()),
                    },
                    header,
                    paths: init.paths.clone(),
                    is_processing: false,
                    processed_count: 0,
                    total_count: init.paths.len(),
                    _body: BodyController::Resize(resize_body),
                };
                (widget, model)
            }
            Mode::Rotate => {
                let rotate_body: Controller<RotateBodyModel> = RotateBodyModel::builder()
                    .launch(())
                    .forward(sender.input_sender(), |msg| match msg {
                        RotateBodyOutput::UpdateAngle(angle) => AppInput::UpdateAngle(angle),
                        RotateBodyOutput::UpdateOutputMode(mode) => {
                            AppInput::UpdateOutputMode(mode)
                        }
                    });
                let widget = rotate_body.widget().clone();
                let model = AppModel {
                    general_config: GeneralConfig {
                        mode: Mode::Rotate,
                        rotation_angle: Some(RotationAngle::Specific(RotationAngleKind::Ninety)),
                        image_size: None,
                        output_mode: OutputMode::NewFile(".rotated".to_owned()),
                    },
                    header,
                    paths: init.paths.clone(),
                    is_processing: false,
                    processed_count: 0,
                    total_count: init.paths.len(),
                    _body: BodyController::Rotate(rotate_body),
                };
                (widget, model)
            }
            Mode::Convert => todo!(),
        };

        let widgets = view_output!();
        widgets.dialog_vbox1.append(&body_widget);

        // ── Window Presentation: Wayland XDG Foreign or X11 Transient ─────────
        if let Some(parent_handle) = init.parent_handle {
            bind_transient_to_wayland_parent(&root, parent_handle);
        } else if let Some(parent_xid) = init.parent_xid {
            crate::x11::bind_transient_to_parent(&root, parent_xid);
        }

        // Close on Escape key press
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed({
            let sender = sender.clone();
            move |_, key, _, _| {
                if key == gtk::gdk::Key::Escape {
                    sender.input(AppInput::Cancel);
                    gtk::glib::Propagation::Stop
                } else {
                    gtk::glib::Propagation::Proceed
                }
            }
        });
        root.add_controller(key_controller);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            AppInput::Cancel => {
                if !self.is_processing {
                    main_application().quit();
                }
            }
            AppInput::Execute => {
                if self.is_processing {
                    return;
                }
                self.is_processing = true;
                self.processed_count = 0;
                self.total_count = self.paths.len();

                // Disable header buttons while processing
                self.header.widget().set_sensitive(false);

                let manipulator: Arc<dyn Manipulator> = match self.general_config.mode {
                    Mode::Resize => {
                        let kind = self
                            .general_config
                            .image_size
                            .unwrap_or(ResizeKind::Percentage(0.5));
                        Arc::new(Resizer(ResizerConfig(kind)))
                    }
                    Mode::Rotate => {
                        let angle = self
                            .general_config
                            .rotation_angle
                            .unwrap_or(RotationAngle::Specific(RotationAngleKind::Ninety));
                        Arc::new(Rotator(RotatorConfig(angle)))
                    }
                    Mode::Convert => {
                        eprintln!(
                            "[nautilus-image-converter] Convert mode is not yet implemented."
                        );
                        main_application().quit();
                        return;
                    }
                };

                let paths = self.paths.clone();
                let output_mode = self.general_config.output_mode.clone();
                let input_sender = sender.input_sender().clone();

                std::thread::spawn(move || {
                    let runtime = tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .expect("Failed to initialize Tokio runtime");

                    runtime.block_on(async move {
                        let completed_counter = Arc::new(AtomicUsize::new(0));

                        let tasks = paths.into_iter().map(|path_str| {
                            let output_mode = output_mode.clone();
                            let manipulator = Arc::clone(&manipulator);
                            let completed_counter = Arc::clone(&completed_counter);
                            let input_sender = input_sender.clone();

                            tokio::task::spawn_blocking(move || {
                                let path = PathBuf::from(&path_str);
                                if let Err(e) =
                                    process_single_image(&path, &output_mode, &*manipulator)
                                {
                                    eprintln!(
                                        "[nautilus-image-converter] Error processing {}: {:?}",
                                        path_str, e
                                    );
                                }
                                let completed =
                                    completed_counter.fetch_add(1, Ordering::SeqCst) + 1;
                                let _ = input_sender.send(AppInput::ProgressStep(completed));
                            })
                        });

                        futures::future::join_all(tasks).await;

                        // Brief pause so the user sees 100% completion
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        let _ = input_sender.send(AppInput::Finished);
                    });
                });
            }
            AppInput::ProgressStep(count) => {
                self.processed_count = count;
            }
            AppInput::Finished => {
                main_application().quit();
            }
            AppInput::UpdateAngle(angle) => {
                self.general_config.rotation_angle = Some(angle);
            }
            AppInput::UpdateImageSize(size) => {
                self.general_config.image_size = Some(size);
            }
            AppInput::UpdateOutputMode(mode) => {
                self.general_config.output_mode = mode;
            }
        }
    }
}

// ── Image processing helpers ──────────────────────────────────────────────────

pub(crate) fn process_single_image(
    path: &Path,
    output_mode: &OutputMode,
    manipulator: &dyn Manipulator,
) -> Result<()> {
    use image::ImageReader;

    let img = ImageReader::open(path)
        .with_context(|| format!("Failed to open image file: {:?}", path))?
        .with_guessed_format()
        .with_context(|| format!("Failed to guess image format: {:?}", path))?
        .decode()
        .with_context(|| format!("Failed to decode image: {:?}", path))?;

    let manipulated = manipulator
        .manipulate_next_image(img)
        .with_context(|| format!("Failed to manipulate image: {:?}", path))?;

    let dest = destination_path(path, output_mode);

    save_image(&manipulated, &dest)?;

    println!("[nautilus-image-converter] Saved: {:?}", dest);
    Ok(())
}

pub(crate) fn destination_path(original: &Path, output_mode: &OutputMode) -> PathBuf {
    match output_mode {
        OutputMode::InPlace => original.to_path_buf(),
        OutputMode::NewFile(suffix) => {
            let parent = original.parent().unwrap_or_else(|| Path::new(""));
            let file_stem = original
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image");
            let extension = original.extension().and_then(|e| e.to_str());
            let new_filename = match extension {
                Some(ext) => format!("{}{}.{}", file_stem, suffix, ext),
                None => format!("{}{}", file_stem, suffix),
            };
            parent.join(new_filename)
        }
    }
}

fn save_image(img: &image::DynamicImage, dest: &Path) -> Result<()> {
    let ext = dest
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if (ext == "jpg" || ext == "jpeg") && img.color().has_alpha() {
        let rgb = img.to_rgb8();
        rgb.save(dest)
            .with_context(|| format!("Failed to save RGB image to {:?}", dest))?;
    } else {
        img.save(dest)
            .with_context(|| format!("Failed to save image to {:?}", dest))?;
    }
    Ok(())
}

/// Binds `window` as a transient child of an exported Wayland parent surface
/// using the XDG Foreign protocol (zxdg_importer_v2).
fn bind_transient_to_wayland_parent(window: &Window, parent_handle: String) {
    use gdk4_wayland::{WaylandToplevel, prelude::*};
    use gtk::prelude::NativeExt;

    window.connect_realize(move |win| {
        let surface = match win.surface() {
            Some(s) => s,
            None => {
                eprintln!("[nautilus-image-converter] Could not obtain window surface on realize.");
                return;
            }
        };

        match surface.downcast::<WaylandToplevel>() {
            Ok(toplevel) => {
                let ok = toplevel.set_transient_for_exported(&parent_handle);
                if ok {
                    println!(
                        "[nautilus-image-converter] Successfully bound Wayland transient-for to exported handle: {}",
                        parent_handle
                    );
                } else {
                    eprintln!(
                        "[nautilus-image-converter] Failed to bind Wayland transient-for to handle: {}",
                        parent_handle
                    );
                }
            }
            Err(_) => {
                eprintln!(
                    "[nautilus-image-converter] Window surface is not a Wayland toplevel (running on X11?)."
                );
            }
        }
    });
}
