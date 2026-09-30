use gtk::{
    Adjustment, Align, Box, CheckButton, DropDown, Entry, Label, Orientation, SpinButton,
    StringList,
    prelude::{BoxExt, CheckButtonExt, EditableExt, EntryExt, OrientableExt, WidgetExt},
};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, component};

use crate::{
    OutputMode,
    manipulators::{resizer::ResizeKind, rotator::{RotationAngle, RotationAngleKind}},
};

pub struct ResizeBodyModel {
    pub cur_percent: u8,
    pub cur_width: u16,
    pub cur_height: u16,
    pub append: String,
}

#[derive(Debug)]
pub enum ResizeBodyInput {
    UpdateCurrentPercent(u8),
    UpdateCurrentWidth(u16),
    UpdateCurrentHeight(u16),
    UpdateAppend(String),
}

#[derive(Debug)]
pub enum ResizeBodyOutput {
    UpdateImageSize(ResizeKind),
    UpdateOutputMode(OutputMode),
}

/// Parse a "WxH" string (e.g. "640x640") into (width, height).
fn parse_wh(s: &str) -> (u32, u32) {
    let mut parts = s.split('x');
    let w = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    let h = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    (w, h)
}

/// Preset sizes matching the DropDown order.
const PRESET_SIZES: &[&str] = &[
    "96x96", "128x128", "640x640", "800x800", "1024x768", "1280x960",
];

#[component(pub)]
impl SimpleComponent for ResizeBodyModel {
    type Init = ();

    type Input = ResizeBodyInput;
    type Output = ResizeBodyOutput;

    view! {
        #[root]
        #[name(vbox1)]
        Box {
            set_hexpand: true,
            set_vexpand: true,
            set_orientation: Orientation::Vertical,
            set_margin_all: 12,
            set_spacing: 12,

            #[name(label2)]
            Label {
                set_valign: Align::Center,
                set_xalign: 0f32,
                set_label: "<b>Image Size</b>",
                set_use_markup: true
            },

            #[name(hbox2)]
            Box {
                set_valign: Align::Center,
                set_spacing: 12,

                #[name(label5)]
                Label {
                    set_halign: Align::Center
                },

                #[name(vbox2)]
                Box {
                    set_hexpand: true,
                    set_orientation: Orientation::Vertical,
                    set_spacing: 6,

                    #[name(hbox4)]
                    Box {
                        set_valign: Align::Center,
                        set_spacing: 6,

                        #[name(default_size_radiobutton)]
                        CheckButton {
                            set_halign: Align::Center,
                            set_label: Some("Select a size"),
                            set_use_underline: true,
                            connect_toggled[sender, comboboxtext_size, width_spinbutton, height_spinbutton, pct_spinbutton, custom_pct_radio_button, custom_size_radiobutton] => move |btn| {
                                if btn.is_active() {
                                    // Preset size selected — read the current dropdown selection.
                                    comboboxtext_size.set_sensitive(true);
                                    pct_spinbutton.set_sensitive(false);
                                    width_spinbutton.set_sensitive(false);
                                    height_spinbutton.set_sensitive(false);
                                    // Unblock the other radio buttons from re-firing.
                                    let _ = custom_pct_radio_button.is_active();
                                    let _ = custom_size_radiobutton.is_active();
                                    let idx = comboboxtext_size.selected() as usize;
                                    let (w, h) = parse_wh(PRESET_SIZES.get(idx).unwrap_or(&"0x0"));
                                    sender.input(ResizeBodyInput::UpdateCurrentWidth(w as u16));
                                    sender.input(ResizeBodyInput::UpdateCurrentHeight(h as u16));
                                    sender.output(ResizeBodyOutput::UpdateImageSize(
                                        ResizeKind::Custom(w, h),
                                    )).unwrap();
                                }
                            }
                        },

                        #[name(comboboxtext_size)]
                        DropDown {
                            set_hexpand: true,
                            set_sensitive: false,
                            set_model: Some(&StringList::new(&["96x96", "128x128", "640x640", "800x800", "1024x768", "1280x960"])),
                            connect_selected_notify[sender] => move |dd| {
                                let idx = dd.selected() as usize;
                                let (w, h) = parse_wh(PRESET_SIZES.get(idx).unwrap_or(&"0x0"));
                                sender.input(ResizeBodyInput::UpdateCurrentWidth(w as u16));
                                sender.input(ResizeBodyInput::UpdateCurrentHeight(h as u16));
                                sender.output(ResizeBodyOutput::UpdateImageSize(
                                    ResizeKind::Custom(w, h),
                                )).unwrap();
                            }
                        },

                        #[name(label9)]
                        Label {
                            set_halign: Align::Center,
                            set_label: "pixels"
                        }
                    },

                    #[name(hbox8)]
                    Box {
                        set_vexpand: true,
                        set_spacing: 6,

                        #[name(custom_pct_radio_button)]
                        CheckButton {
                            set_halign: Align::Center,
                            set_label: Some("Scale:"),
                            set_use_underline: true,
                            set_group: Some(&default_size_radiobutton),
                            set_active: true,
                            connect_toggled[sender, pct_spinbutton, comboboxtext_size, width_spinbutton, height_spinbutton] => move |btn| {
                                if btn.is_active() {
                                    pct_spinbutton.set_sensitive(true);
                                    comboboxtext_size.set_sensitive(false);
                                    width_spinbutton.set_sensitive(false);
                                    height_spinbutton.set_sensitive(false);
                                    let pct = pct_spinbutton.value() as u8;
                                    sender.input(ResizeBodyInput::UpdateCurrentPercent(pct));
                                    sender.output(ResizeBodyOutput::UpdateImageSize(
                                        ResizeKind::Percentage(pct as f32 / 100.0),
                                    )).unwrap();
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
                                let pct = spin.value() as u8;
                                sender.input(ResizeBodyInput::UpdateCurrentPercent(pct));
                                sender.output(ResizeBodyOutput::UpdateImageSize(
                                    ResizeKind::Percentage(pct as f32 / 100.0),
                                )).unwrap();
                            }
                        },

                        #[name(label15)]
                        Label {
                            set_halign: Align::Center,
                            set_label: "percent"
                        }
                    },

                    #[name(hbox5)]
                    Box {
                        set_vexpand: true,
                        set_spacing: 6,

                        #[name(custom_size_radiobutton)]
                        CheckButton{
                            set_halign: Align::Center,
                            set_label: Some("Custom size:"),
                            set_use_underline: true,
                            set_group: Some(&default_size_radiobutton),
                            connect_toggled[sender, width_spinbutton, height_spinbutton, pct_spinbutton, comboboxtext_size] => move |btn| {
                                if btn.is_active() {
                                    width_spinbutton.set_sensitive(true);
                                    height_spinbutton.set_sensitive(true);
                                    pct_spinbutton.set_sensitive(false);
                                    comboboxtext_size.set_sensitive(false);
                                    let w = width_spinbutton.value() as u32;
                                    let h = height_spinbutton.value() as u32;
                                    sender.input(ResizeBodyInput::UpdateCurrentWidth(w as u16));
                                    sender.input(ResizeBodyInput::UpdateCurrentHeight(h as u16));
                                    sender.output(ResizeBodyOutput::UpdateImageSize(
                                        ResizeKind::Custom(w, h),
                                    )).unwrap();
                                }
                            }
                        },

                        #[name(label10)]
                        Label {
                            set_halign: Align::Center,
                            set_label: "Width:"
                        },

                        #[name(width_spinbutton)]
                        SpinButton {
                            set_hexpand: true,
                            set_sensitive: false,
                            set_adjustment: &Adjustment::new(1000f64, 1f64, 9999f64, 1f64, 10f64, 0f64),
                            set_climb_rate: 1f64,
                            connect_value_changed[sender, height_spinbutton] => move |spin| {
                                let w = spin.value() as u32;
                                let h = height_spinbutton.value() as u32;
                                sender.input(ResizeBodyInput::UpdateCurrentWidth(w as u16));
                                sender.output(ResizeBodyOutput::UpdateImageSize(
                                    ResizeKind::Custom(w, h),
                                )).unwrap();
                            }
                        },

                        #[name(label11)]
                        Label {
                            set_halign: Align::Center,
                            set_label: "Height:"
                        },

                        #[name(height_spinbutton)]
                        SpinButton {
                            set_hexpand: true,
                            set_sensitive: false,
                            set_adjustment: &Adjustment::new(1000f64, 1f64, 9999f64, 1f64, 10f64, 0f64),
                            set_climb_rate: 1f64,
                            connect_value_changed[sender, width_spinbutton] => move |spin| {
                                let h = spin.value() as u32;
                                let w = width_spinbutton.value() as u32;
                                sender.input(ResizeBodyInput::UpdateCurrentHeight(h as u16));
                                sender.output(ResizeBodyOutput::UpdateImageSize(
                                    ResizeKind::Custom(w, h),
                                )).unwrap();
                            }
                        },

                        #[name(label14)]
                        Label {
                            set_halign: Align::Center,
                            set_label: "pixels"
                        }
                    }
                },
            },

            #[name(label3)]
            Label {
                set_valign: Align::Center,
                set_xalign: 0f32,
                set_label: "<b>Filename</b>",
                set_use_markup: true
            },

            #[name(hbox6)]
            Box {
                set_valign: Align::Center,
                set_spacing: 12,

                #[name(label12)]
                Label {
                    set_halign: Align::Center
                },

                #[name(vbox3)]
                Box {
                    set_hexpand: true,
                    set_orientation: Orientation::Vertical,
                    set_spacing: 6,

                    #[name(hbox7)]
                    Box {
                        set_vexpand: true,
                        set_spacing: 6,

                        #[name(append_radiobutton)]
                        CheckButton {
                            set_halign: Align::Center,
                            set_active: true,
                            set_label: Some("Append"),
                            set_use_underline: true,
                            connect_toggled[sender, name_entry] => move |btn| {
                                if btn.is_active() {
                                    name_entry.set_sensitive(true);
                                    let text = name_entry.text().to_string();
                                    sender.input(ResizeBodyInput::UpdateAppend(text.clone()));
                                    sender.output(ResizeBodyOutput::UpdateOutputMode(
                                        OutputMode::NewFile(text),
                                    )).unwrap();
                                }
                            }
                        },

                        #[name(name_entry)]
                        Entry {
                            set_hexpand: true,
                            set_text: ".resized",
                            set_activates_default: true,
                            connect_changed[sender] => move |entry| {
                                let text = entry.text().to_string();
                                sender.input(ResizeBodyInput::UpdateAppend(text.clone()));
                                sender.output(ResizeBodyOutput::UpdateOutputMode(
                                    OutputMode::NewFile(text),
                                )).unwrap();
                            }
                        },

                        #[name(label13)]
                        Label {
                            set_halign: Align::Center,
                            set_label: "to file title"
                        }
                    },

                    #[name(inplace_radiobutton)]
                    CheckButton {
                        set_valign: Align::Center,
                        set_label: Some("Resize in place"),
                        set_use_underline: true,
                        set_group: Some(&append_radiobutton),
                        connect_toggled[sender, name_entry] => move |btn| {
                            if btn.is_active() {
                                name_entry.set_sensitive(false);
                                sender.output(ResizeBodyOutput::UpdateOutputMode(
                                    OutputMode::InPlace,
                                )).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ResizeBodyModel {
            cur_percent: 50,
            cur_width: 1000,
            cur_height: 1000,
            append: ".resized".to_owned(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ResizeBodyInput::UpdateAppend(append) => {
                self.append = append;
            }
            ResizeBodyInput::UpdateCurrentPercent(percent) => self.cur_percent = percent,
            ResizeBodyInput::UpdateCurrentHeight(height) => {
                self.cur_height = height;
            }
            ResizeBodyInput::UpdateCurrentWidth(width) => {
                self.cur_width = width;
            }
        };
    }
}

pub struct RotateBodyModel {
    pub custom_angle: u8,
    pub append: String,
}

#[derive(Debug)]
pub enum RotateBodyInput {
    UpdateCustomAngle(u8),
    UpdateAppend(String),
}

#[derive(Debug)]
pub enum RotateBodyOutput {
    UpdateAngle(RotationAngle),
    UpdateOutputMode(OutputMode),
}

/// Map a DropDown index (0..=2) to a `RotationAngleKind`.
fn angle_kind_from_index(idx: u32) -> RotationAngleKind {
    match idx {
        0 => RotationAngleKind::Ninety,
        1 => RotationAngleKind::HundredEighty,
        _ => RotationAngleKind::TwoHundredSeventy,
    }
}

#[component(pub)]
impl SimpleComponent for RotateBodyModel {
    type Init = ();

    type Input = RotateBodyInput;
    type Output = RotateBodyOutput;

    view! {
        #[root]
        #[name(vbox1)]
        Box {
            set_hexpand: true,
            set_vexpand: true,
            set_orientation: Orientation::Vertical,
            set_margin_all: 12,
            set_spacing: 12,

            #[name(label2)]
            Label {
                set_valign: Align::Center,
                set_xalign: 0f32,
                set_label: "<b>Image Rotation</b>",
                set_use_markup: true
            },

            #[name(hbox2)]
            Box {
                set_valign: Align::Center,
                set_spacing: 12,

                #[name(label5)]
                Label {
                    set_halign: Align::Center
                },

                #[name(vbox2)]
                Box {
                    set_orientation: Orientation::Vertical,
                    set_spacing: 6,

                    #[name(hbox4)]
                    Box {
                        set_valign: Align::Center,
                        set_spacing: 6,

                        #[name(default_angle_radiobutton)]
                        CheckButton {
                            set_halign: Align::Center,
                            set_label: Some("Select an angle:"),
                            set_use_underline: true,
                            set_active: true,
                            connect_toggled[sender, angle_combobox, angle_spinbutton] => move |btn| {
                                if btn.is_active() {
                                    angle_combobox.set_sensitive(true);
                                    angle_spinbutton.set_sensitive(false);
                                    let kind = angle_kind_from_index(angle_combobox.selected());
                                    sender.output(RotateBodyOutput::UpdateAngle(
                                        RotationAngle::Specific(kind),
                                    )).unwrap();
                                }
                            }
                        },

                        #[name(angle_combobox)]
                        DropDown {
                            set_hexpand: true,
                            set_model: Some(&StringList::new(&["90° clockwise", "180°", "90° counter-clockwise"])),
                            connect_selected_notify[sender] => move |dd| {
                                let kind = angle_kind_from_index(dd.selected());
                                sender.output(RotateBodyOutput::UpdateAngle(
                                    RotationAngle::Specific(kind),
                                )).unwrap();
                            }
                        }
                    },

                    #[name(hbox8)]
                    Box {
                        set_spacing: 6,

                        #[name(custom_angle_radiobutton)]
                        CheckButton {
                            set_halign: Align::Center,
                            set_label: Some("Custom angle:"),
                            set_use_underline: true,
                            set_group: Some(&default_angle_radiobutton),
                            connect_toggled[sender, angle_spinbutton, angle_combobox] => move |btn| {
                                if btn.is_active() {
                                    angle_spinbutton.set_sensitive(true);
                                    angle_combobox.set_sensitive(false);
                                    let deg = angle_spinbutton.value() as u32;
                                    sender.input(RotateBodyInput::UpdateCustomAngle(deg as u8));
                                    sender.output(RotateBodyOutput::UpdateAngle(
                                        RotationAngle::Custom(deg),
                                    )).unwrap();
                                }
                            }
                        },

                        #[name(angle_spinbutton)]
                        SpinButton {
                            set_sensitive: false,
                            set_adjustment: &Adjustment::new(90f64, 1f64, 360f64, 1f64, 45f64, 0f64),
                            set_climb_rate: 1f64,
                            set_numeric: true,
                            connect_value_changed[sender] => move |spin| {
                                let deg = spin.value() as u32;
                                sender.input(RotateBodyInput::UpdateCustomAngle(deg as u8));
                                sender.output(RotateBodyOutput::UpdateAngle(
                                    RotationAngle::Custom(deg),
                                )).unwrap();
                            }
                        },

                        #[name(label15)]
                        Label {
                            set_halign: Align::Center,
                            set_label: "degrees clockwise"
                        }
                    }
                }
            },

            #[name(label3)]
            Label {
                set_valign: Align::Center,
                set_xalign: 0f32,
                set_label: "<b>Filename</b>",
                set_use_markup: true,
            },

            #[name(hbox6)]
            Box {
                set_valign: Align::Center,
                set_spacing: 12,

                #[name(label12)]
                Label {
                    set_halign: Align::Center
                },

                #[name(vbox3)]
                Box {
                    set_orientation: Orientation::Vertical,
                    set_spacing: 6,

                    #[name(hbox7)]
                    Box {
                        set_spacing: 6,

                        #[name(append_radiobutton)]
                        CheckButton {
                            set_halign: Align::Center,
                            set_active: true,
                            set_label: Some("Append"),
                            set_use_underline: true,
                            connect_toggled[sender, name_entry] => move |btn| {
                                if btn.is_active() {
                                    name_entry.set_sensitive(true);
                                    let text = name_entry.text().to_string();
                                    sender.input(RotateBodyInput::UpdateAppend(text.clone()));
                                    sender.output(RotateBodyOutput::UpdateOutputMode(
                                        OutputMode::NewFile(text),
                                    )).unwrap();
                                }
                            }
                        },

                        #[name(name_entry)]
                        Entry {
                            set_hexpand: true,
                            set_text: ".rotated",
                            connect_changed[sender] => move |entry| {
                                let text = entry.text().to_string();
                                sender.input(RotateBodyInput::UpdateAppend(text.clone()));
                                sender.output(RotateBodyOutput::UpdateOutputMode(
                                    OutputMode::NewFile(text),
                                )).unwrap();
                            }
                        },

                        #[name(label13)]
                        Label {
                            set_halign: Align::Center,
                            set_label: "to file title"
                        }
                    },

                    #[name(inplace_radiobutton)]
                    CheckButton {
                        set_valign: Align::Center,
                        set_label: Some("Rotate in place"),
                        set_use_underline: true,
                        set_group: Some(&append_radiobutton),
                        connect_toggled[sender, name_entry] => move |btn| {
                            if btn.is_active() {
                                name_entry.set_sensitive(false);
                                sender.output(RotateBodyOutput::UpdateOutputMode(
                                    OutputMode::InPlace,
                                )).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RotateBodyModel {
            custom_angle: 90,
            append: ".rotated".to_owned(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            RotateBodyInput::UpdateAppend(append) => {
                self.append = append;
            }
            RotateBodyInput::UpdateCustomAngle(angle) => {
                self.custom_angle = angle;
            }
        }
    }
}
