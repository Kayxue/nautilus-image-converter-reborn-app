use gtk::{
    Adjustment, Align, Box, CheckButton, DropDown, Entry, Label, Orientation, SpinButton,
    StringList,
    prelude::{BoxExt, CheckButtonExt, EditableExt, ObjectExt, OrientableExt, WidgetExt},
};
use image::ImageFormat;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, component,
};

use super::ConfigType;
use crate::{
    manipulators::{
        converter::ConverterConfig,
        resizer::{ResizeKind, ResizerConfig},
        rotator::{RotationAngle, RotationAngleKind, RotatorConfig},
    },
    output::OutputMode,
};

/// Output shared by all window bodies.
#[derive(Debug)]
pub enum BodyOutput {
    UpdateConfig(ConfigType),
    UpdateOutputMode(OutputMode),
}

impl BodyOutput {
    fn resize(kind: ResizeKind) -> Self {
        Self::UpdateConfig(ConfigType::Resize(ResizerConfig(kind)))
    }

    fn rotate(angle: RotationAngle) -> Self {
        Self::UpdateConfig(ConfigType::Rotate(RotatorConfig(angle)))
    }

    fn convert(format: ImageFormat) -> Self {
        Self::UpdateConfig(ConfigType::Convert(ConverterConfig(format)))
    }
}

/// Makes `target` sensitive only while `radio` is active.
fn bind_sensitive(radio: &CheckButton, target: &impl gtk::prelude::IsA<gtk::Widget>) {
    radio
        .bind_property("active", target, "sensitive")
        .sync_create()
        .build();
}

// ── Filename section (shared by Resize and Rotate) ───────────────────────────

pub struct FilenameInit {
    /// Default suffix appended to the file title.
    pub suffix: &'static str,
    /// Label of the "overwrite in place" radio button.
    pub in_place_label: &'static str,
}

pub struct FilenameModel;

#[component(pub)]
impl SimpleComponent for FilenameModel {
    type Init = FilenameInit;

    type Input = ();
    type Output = OutputMode;

    view! {
        #[root]
        Box {
            set_orientation: Orientation::Vertical,
            set_spacing: 12,

            Label {
                set_xalign: 0f32,
                set_label: "<b>Filename</b>",
                set_use_markup: true,
            },

            Box {
                set_orientation: Orientation::Vertical,
                set_spacing: 6,
                set_margin_start: 12,

                Box {
                    set_spacing: 6,

                    #[name(append_radiobutton)]
                    CheckButton {
                        set_active: true,
                        set_label: Some("Append"),
                        set_use_underline: true,
                        connect_toggled[sender, name_entry] => move |btn| {
                            if btn.is_active() {
                                sender.output(OutputMode::NewFile(name_entry.text().into())).unwrap();
                            }
                        }
                    },

                    #[name(name_entry)]
                    Entry {
                        set_hexpand: true,
                        set_text: init.suffix,
                        connect_changed[sender] => move |entry| {
                            sender.output(OutputMode::NewFile(entry.text().into())).unwrap();
                        }
                    },

                    Label {
                        set_label: "to file title",
                    }
                },

                CheckButton {
                    set_label: Some(init.in_place_label),
                    set_use_underline: true,
                    set_group: Some(&append_radiobutton),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(OutputMode::InPlace).unwrap();
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = FilenameModel;
        let widgets = view_output!();
        bind_sensitive(&widgets.append_radiobutton, &widgets.name_entry);
        ComponentParts { model, widgets }
    }
}

/// Launches the filename section and forwards its output through `sender`.
fn launch_filename<C>(init: FilenameInit, sender: &ComponentSender<C>) -> Controller<FilenameModel>
where
    C: Component<Output = BodyOutput>,
{
    FilenameModel::builder()
        .launch(init)
        .forward(sender.output_sender(), BodyOutput::UpdateOutputMode)
}

// ── Resize ───────────────────────────────────────────────────────────────────

/// Preset sizes (width, height) in DropDown order.
const PRESET_SIZES: &[(u32, u32)] = &[
    (96, 96),
    (128, 128),
    (640, 640),
    (800, 800),
    (1024, 768),
    (1280, 960),
];

fn preset_size_list() -> StringList {
    let list = StringList::new(&[]);
    for (w, h) in PRESET_SIZES {
        list.append(&format!("{w}x{h}"));
    }
    list
}

fn preset_size(idx: u32) -> ResizeKind {
    let (w, h) = PRESET_SIZES.get(idx as usize).copied().unwrap_or_default();
    ResizeKind::Custom(w, h)
}

pub struct ResizeBodyModel {
    filename: Controller<FilenameModel>,
}

#[component(pub)]
impl SimpleComponent for ResizeBodyModel {
    type Init = ();

    type Input = ();
    type Output = BodyOutput;

    view! {
        #[root]
        Box {
            set_hexpand: true,
            set_vexpand: false,
            set_orientation: Orientation::Vertical,
            set_margin_all: 12,
            set_spacing: 12,

            Label {
                set_xalign: 0f32,
                set_label: "<b>Image Size</b>",
                set_use_markup: true
            },

            Box {
                set_hexpand: true,
                set_orientation: Orientation::Vertical,
                set_spacing: 6,
                set_margin_start: 12,

                Box {
                    set_spacing: 6,

                    #[name(default_size_radiobutton)]
                    CheckButton {
                        set_label: Some("Select a size"),
                        set_use_underline: true,
                        connect_toggled[sender, comboboxtext_size] => move |btn| {
                            if btn.is_active() {
                                sender.output(BodyOutput::resize(preset_size(comboboxtext_size.selected()))).unwrap();
                            }
                        }
                    },

                    #[name(comboboxtext_size)]
                    DropDown {
                        set_hexpand: true,
                        set_model: Some(&preset_size_list()),
                        connect_selected_notify[sender] => move |dd| {
                            sender.output(BodyOutput::resize(preset_size(dd.selected()))).unwrap();
                        }
                    },

                    Label {
                        set_label: "pixels"
                    }
                },

                Box {
                    set_spacing: 6,

                    #[name(custom_pct_radio_button)]
                    CheckButton {
                        set_label: Some("Scale:"),
                        set_use_underline: true,
                        set_group: Some(&default_size_radiobutton),
                        set_active: true,
                        connect_toggled[sender, pct_spinbutton] => move |btn| {
                            if btn.is_active() {
                                sender.output(BodyOutput::resize(percentage(&pct_spinbutton))).unwrap();
                            }
                        }
                    },

                    #[name(pct_spinbutton)]
                    SpinButton {
                        set_hexpand: true,
                        set_adjustment: &Adjustment::new(50f64, 1f64, 100f64, 1f64, 10f64, 0f64),
                        set_climb_rate: 1f64,
                        set_numeric: true,
                        connect_value_changed[sender] => move |spin| {
                            sender.output(BodyOutput::resize(percentage(spin))).unwrap();
                        }
                    },

                    Label {
                        set_label: "percent"
                    }
                },

                Box {
                    set_spacing: 6,

                    #[name(custom_size_radiobutton)]
                    CheckButton {
                        set_label: Some("Custom size:"),
                        set_use_underline: true,
                        set_group: Some(&default_size_radiobutton),
                        connect_toggled[sender, width_spinbutton, height_spinbutton] => move |btn| {
                            if btn.is_active() {
                                sender.output(BodyOutput::resize(custom_size(&width_spinbutton, &height_spinbutton))).unwrap();
                            }
                        }
                    },

                    Label {
                        set_label: "Width:"
                    },

                    #[name(width_spinbutton)]
                    SpinButton {
                        set_hexpand: true,
                        set_adjustment: &Adjustment::new(1000f64, 1f64, 9999f64, 1f64, 10f64, 0f64),
                        set_climb_rate: 1f64,
                        connect_value_changed[sender, height_spinbutton] => move |spin| {
                            sender.output(BodyOutput::resize(custom_size(spin, &height_spinbutton))).unwrap();
                        }
                    },

                    Label {
                        set_label: "Height:"
                    },

                    #[name(height_spinbutton)]
                    SpinButton {
                        set_hexpand: true,
                        set_adjustment: &Adjustment::new(1000f64, 1f64, 9999f64, 1f64, 10f64, 0f64),
                        set_climb_rate: 1f64,
                        connect_value_changed[sender, width_spinbutton] => move |spin| {
                            sender.output(BodyOutput::resize(custom_size(&width_spinbutton, spin))).unwrap();
                        }
                    },

                    Label {
                        set_label: "pixels"
                    }
                }
            },

            append: model.filename.widget(),
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let filename = launch_filename(
            FilenameInit {
                suffix: ".resized",
                in_place_label: "Resize in place",
            },
            &sender,
        );
        let model = ResizeBodyModel { filename };

        let widgets = view_output!();
        bind_sensitive(&widgets.default_size_radiobutton, &widgets.comboboxtext_size);
        bind_sensitive(&widgets.custom_pct_radio_button, &widgets.pct_spinbutton);
        bind_sensitive(&widgets.custom_size_radiobutton, &widgets.width_spinbutton);
        bind_sensitive(&widgets.custom_size_radiobutton, &widgets.height_spinbutton);

        ComponentParts { model, widgets }
    }
}

fn percentage(spin: &SpinButton) -> ResizeKind {
    ResizeKind::Percentage(spin.value() as u8 as f32 / 100.0)
}

fn custom_size(width: &SpinButton, height: &SpinButton) -> ResizeKind {
    ResizeKind::Custom(width.value() as u32, height.value() as u32)
}

// ── Rotate ───────────────────────────────────────────────────────────────────

/// Map a DropDown index (0..=2) to a `RotationAngleKind`.
fn angle_kind_from_index(idx: u32) -> RotationAngleKind {
    match idx {
        0 => RotationAngleKind::Ninety,
        1 => RotationAngleKind::HundredEighty,
        _ => RotationAngleKind::TwoHundredSeventy,
    }
}

pub struct RotateBodyModel {
    filename: Controller<FilenameModel>,
}

#[component(pub)]
impl SimpleComponent for RotateBodyModel {
    type Init = ();

    type Input = ();
    type Output = BodyOutput;

    view! {
        #[root]
        Box {
            set_hexpand: true,
            set_vexpand: false,
            set_orientation: Orientation::Vertical,
            set_margin_all: 12,
            set_spacing: 12,

            Label {
                set_xalign: 0f32,
                set_label: "<b>Image Rotation</b>",
                set_use_markup: true
            },

            Box {
                set_orientation: Orientation::Vertical,
                set_spacing: 6,
                set_margin_start: 12,

                Box {
                    set_spacing: 6,

                    #[name(default_angle_radiobutton)]
                    CheckButton {
                        set_label: Some("Select an angle:"),
                        set_use_underline: true,
                        set_active: true,
                        connect_toggled[sender, angle_combobox] => move |btn| {
                            if btn.is_active() {
                                let kind = angle_kind_from_index(angle_combobox.selected());
                                sender.output(BodyOutput::rotate(RotationAngle::Specific(kind))).unwrap();
                            }
                        }
                    },

                    #[name(angle_combobox)]
                    DropDown {
                        set_hexpand: true,
                        set_model: Some(&StringList::new(&["90° clockwise", "180°", "90° counter-clockwise"])),
                        connect_selected_notify[sender] => move |dd| {
                            let kind = angle_kind_from_index(dd.selected());
                            sender.output(BodyOutput::rotate(RotationAngle::Specific(kind))).unwrap();
                        }
                    }
                },

                Box {
                    set_spacing: 6,

                    #[name(custom_angle_radiobutton)]
                    CheckButton {
                        set_label: Some("Custom angle:"),
                        set_use_underline: true,
                        set_group: Some(&default_angle_radiobutton),
                        connect_toggled[sender, angle_spinbutton] => move |btn| {
                            if btn.is_active() {
                                let deg = angle_spinbutton.value() as u32;
                                sender.output(BodyOutput::rotate(RotationAngle::Custom(deg))).unwrap();
                            }
                        }
                    },

                    #[name(angle_spinbutton)]
                    SpinButton {
                        set_hexpand: true,
                        set_adjustment: &Adjustment::new(90f64, 1f64, 360f64, 1f64, 45f64, 0f64),
                        set_climb_rate: 1f64,
                        set_numeric: true,
                        connect_value_changed[sender] => move |spin| {
                            let deg = spin.value() as u32;
                            sender.output(BodyOutput::rotate(RotationAngle::Custom(deg))).unwrap();
                        }
                    },

                    Label {
                        set_label: "degrees clockwise"
                    }
                }
            },

            append: model.filename.widget(),
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let filename = launch_filename(
            FilenameInit {
                suffix: ".rotated",
                in_place_label: "Rotate in place",
            },
            &sender,
        );
        let model = RotateBodyModel { filename };

        let widgets = view_output!();
        bind_sensitive(&widgets.default_angle_radiobutton, &widgets.angle_combobox);
        bind_sensitive(&widgets.custom_angle_radiobutton, &widgets.angle_spinbutton);

        ComponentParts { model, widgets }
    }
}

// ── Convert ──────────────────────────────────────────────────────────────────

/// All image formats that can be written, paired with their primary file
/// extension. The index in this list matches the index in the dropdown.
pub(crate) fn writable_formats() -> Vec<(ImageFormat, &'static str)> {
    ImageFormat::all()
        .filter(|f| f.writing_enabled())
        .filter_map(|f| f.extensions_str().first().map(|ext| (f, *ext)))
        .collect()
}

pub struct ConvertBodyModel {
    /// Index of the initially selected format in [`writable_formats`].
    initial_format_index: u32,
}

#[component(pub)]
impl SimpleComponent for ConvertBodyModel {
    /// The initially selected target format.
    type Init = ImageFormat;

    type Input = ();
    type Output = BodyOutput;

    view! {
        #[root]
        Box {
            set_hexpand: true,
            set_vexpand: false,
            set_orientation: Orientation::Vertical,
            set_margin_all: 12,
            set_spacing: 6,

            Box {
                set_valign: Align::Center,
                set_spacing: 6,

                Label {
                    set_label: "Convert images to: ",
                },

                DropDown {
                    set_hexpand: true,
                    set_model: Some(&StringList::new(
                        &writable_formats().iter().map(|(_, ext)| *ext).collect::<Vec<_>>(),
                    )),
                    set_selected: model.initial_format_index,
                    connect_selected_notify[sender] => move |dd| {
                        if let Some((format, _)) = writable_formats().get(dd.selected() as usize) {
                            sender.output(BodyOutput::convert(*format)).unwrap();
                        }
                    }
                },
            },

            Label {
                set_xalign: 0f32,
                set_label: "<b>Original files (if converted successfully)</b>",
                set_use_markup: true,
            },

            Box {
                set_orientation: Orientation::Vertical,
                set_spacing: 6,
                set_margin_start: 12,

                // "Keep" writes `name.<new ext>` next to the original;
                // "Remove" replaces the original.
                #[name(keep_radiobutton)]
                CheckButton {
                    set_active: true,
                    set_label: Some("Keep original file(s)"),
                    set_use_underline: true,
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(BodyOutput::UpdateOutputMode(OutputMode::NewFile(String::new()))).unwrap();
                        }
                    }
                },

                CheckButton {
                    set_label: Some("Remove original file(s)"),
                    set_use_underline: true,
                    set_group: Some(&keep_radiobutton),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(BodyOutput::UpdateOutputMode(OutputMode::InPlace)).unwrap();
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let initial_format_index = writable_formats()
            .iter()
            .position(|(format, _)| *format == init)
            .unwrap_or(0) as u32;
        let model = ConvertBodyModel {
            initial_format_index,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
