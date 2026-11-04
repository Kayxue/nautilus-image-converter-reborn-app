use image::ImageFormat;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use strum_macros::Display;

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
    Mode,
    manipulators::{
        Manipulator,
        converter::{Converter, ConverterConfig},
        resizer::{ResizeKind, Resizer, ResizerConfig},
        rotator::{RotationAngle, RotationAngleKind, Rotator, RotatorConfig},
    },
    output::{OutputMode, save_image},
    window::window_body::{BodyOutput, ConvertBodyModel, ResizeBodyModel, RotateBodyModel},
};

mod window_body;

#[cfg(test)]
pub(crate) use window_body::writable_formats;

pub struct Initializer {
    pub mode: Mode,
    pub paths: Vec<PathBuf>,
    /// XID of the parent (Nautilus) window passed by the extension (X11 only).
    pub parent_xid: Option<u64>,
    /// Exported handle token of the parent (Nautilus) window (Wayland XDG Foreign).
    pub parent_handle: Option<String>,
}

#[derive(Debug, Display)]
#[strum(serialize_all = "title_case")]
pub enum ConfigType {
    Rotate(RotatorConfig),
    Resize(ResizerConfig),
    Convert(ConverterConfig),
}

impl From<Mode> for ConfigType {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Resize => ConfigType::Resize(ResizerConfig(ResizeKind::Percentage(0.5))),
            Mode::Rotate => ConfigType::Rotate(RotatorConfig(RotationAngle::Specific(
                RotationAngleKind::Ninety,
            ))),
            Mode::Convert => ConfigType::Convert(ConverterConfig(ImageFormat::Png)),
        }
    }
}

pub struct GeneralConfig {
    pub config: ConfigType,
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
    Convert(Controller<ConvertBodyModel>),
}

pub struct AppModel {
    general_config: GeneralConfig,
    header: Controller<HeaderModel>,
    paths: Vec<PathBuf>,
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
    UpdateConfig(ConfigType),
    UpdateOutputMode(OutputMode),
    ProgressStep(usize),
    Finished,
}

impl From<BodyOutput> for AppInput {
    fn from(output: BodyOutput) -> Self {
        match output {
            BodyOutput::UpdateConfig(config) => AppInput::UpdateConfig(config),
            BodyOutput::UpdateOutputMode(mode) => AppInput::UpdateOutputMode(mode),
        }
    }
}

#[component(pub)]
impl SimpleComponent for AppModel {
    type Init = Initializer;

    type Input = AppInput;
    type Output = ();

    view! {
        #[root]
        Window {
            set_title: Some(&format!("{} Images", model.general_config.config)),
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

        let config: ConfigType = init.mode.clone().into();

        // Launch the mode-specific body. The default output mode must match
        // the initial state of the body's radio buttons.
        let input = sender.input_sender();
        let (body_widget, body, output_mode) = match &config {
            ConfigType::Resize(_) => {
                let body = ResizeBodyModel::builder().launch(()).forward(input, AppInput::from);
                (
                    body.widget().clone(),
                    BodyController::Resize(body),
                    OutputMode::NewFile(".resized".to_owned()),
                )
            }
            ConfigType::Rotate(_) => {
                let body = RotateBodyModel::builder().launch(()).forward(input, AppInput::from);
                (
                    body.widget().clone(),
                    BodyController::Rotate(body),
                    OutputMode::NewFile(".rotated".to_owned()),
                )
            }
            ConfigType::Convert(ConverterConfig(format)) => {
                let body = ConvertBodyModel::builder()
                    .launch(*format)
                    .forward(input, AppInput::from);
                (
                    body.widget().clone(),
                    BodyController::Convert(body),
                    // "Keep original file(s)" is selected by default.
                    OutputMode::NewFile(String::new()),
                )
            }
        };

        let model = AppModel {
            general_config: GeneralConfig {
                config,
                output_mode,
            },
            header,
            total_count: init.paths.len(),
            paths: init.paths,
            is_processing: false,
            processed_count: 0,
            _body: body,
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

                // Disable header buttons while processing
                self.header.widget().set_sensitive(false);

                let manipulator: Arc<dyn Manipulator> = match &self.general_config.config {
                    ConfigType::Resize(resize_config) => Arc::new(Resizer(*resize_config)),
                    ConfigType::Rotate(rotate_config) => Arc::new(Rotator(*rotate_config)),
                    ConfigType::Convert(convert_config) => {
                        Arc::new(Converter(convert_config.clone()))
                    }
                };

                let paths = self.paths.clone();
                let output_mode = self.general_config.output_mode.clone();
                let input_sender = sender.input_sender().clone();

                std::thread::spawn(move || {
                    use rayon::prelude::*;

                    let completed_counter = AtomicUsize::new(0);

                    paths.into_par_iter().for_each(|path| {
                        if let Err(e) = process_single_image(&path, &output_mode, &*manipulator) {
                            eprintln!(
                                "[nautilus-image-converter] Error processing {:?}: {:?}",
                                path, e
                            );
                        }
                        let completed = completed_counter.fetch_add(1, Ordering::Relaxed) + 1;
                        let _ = input_sender.send(AppInput::ProgressStep(completed));
                    });

                    // Brief pause so the user sees 100% completion
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    let _ = input_sender.send(AppInput::Finished);
                });
            }
            AppInput::ProgressStep(count) => {
                self.processed_count = count;
            }
            AppInput::Finished => {
                main_application().quit();
            }
            AppInput::UpdateConfig(config) => {
                self.general_config.config = config;
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

    let target_ext = manipulator.target_extension(path);
    let dest = output_mode.destination(path, target_ext.as_deref());

    save_image(&manipulated, &dest)?;
    println!("[nautilus-image-converter] Saved: {:?}", dest);

    // Only reached after a successful save, so the original is never lost.
    if output_mode.removes_original(path, &dest) {
        std::fs::remove_file(path)
            .with_context(|| format!("Failed to remove original file: {:?}", path))?;
        println!("[nautilus-image-converter] Removed original: {:?}", path);
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
