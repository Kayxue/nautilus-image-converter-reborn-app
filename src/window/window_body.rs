use gtk::{
    Adjustment, Align, Box, CheckButton, DropDown, Entry, Label, Orientation, SpinButton, StringList, prelude::{BoxExt, CheckButtonExt, EditableExt, EntryExt, OrientableExt, WidgetExt}
};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, component};

use crate::{
    OutputMode,
    manipulators::{resizer::ResizeKind, rotator::RotationAngle},
};

pub struct ResizeBodyModel {
    pub cur_percent: u8,
    pub cur_width: u16,
    pub cur_height: u16,
    pub append: String
}

#[derive(Debug)]
pub enum ResizeBodyOutput {
    UpdateImageSize(ResizeKind),
    UpdateOutputMode(OutputMode),
}

#[component(pub)]
impl SimpleComponent for ResizeBodyModel {
    type Init = ();

    type Input = ();
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
                            set_use_underline: true
                        },

                        #[name(comboboxtext_size)]
                        DropDown {
                            set_hexpand: true,
                            set_model: Some(&StringList::new(&["96x96", "128x128", "640x640", "800x800", "1024x768", "1280x960"]))
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
                            set_active: true
                        },

                        #[name(pct_spinbutton)]
                        SpinButton {
                            set_hexpand: true,
                            set_adjustment: &Adjustment::new(50f64, 1f64, 100f64, 1f64, 10f64, 0f64),
                            set_climb_rate: 1f64,
                            set_numeric: true
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
                            set_group: Some(&default_size_radiobutton)
                        },

                        #[name(label10)]
                        Label {
                            set_halign: Align::Center,
                            set_label: "Width:"
                        },

                        #[name(width_spinbutton)]
                        SpinButton {
                            set_hexpand: true,
                            set_adjustment: &Adjustment::new(1000f64, 1f64, 9999f64, 1f64, 10f64, 0f64),
                            set_climb_rate: 1f64
                        },

                        #[name(label11)]
                        Label {
                            set_halign: Align::Center,
                            set_label: "Height:"
                        },

                        #[name(height_spinbutton)]
                        SpinButton {
                            set_hexpand: true,
                            set_adjustment: &Adjustment::new(1000f64, 1f64, 9999f64, 1f64, 10f64, 0f64),
                            set_climb_rate: 1f64
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
                            set_use_underline: true
                        },

                        #[name(name_entry)]
                        Entry {
                            set_hexpand: true,
                            set_text: ".resized",
                            set_activates_default: true
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
                        set_group: Some(&append_radiobutton)
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
            append: ".resized".to_owned()
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

pub struct RotateBodyModel {
    pub custom_angle: u8,
    pub append: String
}

#[derive(Debug)]
pub enum RotateBodyOutput {
    UpdateAngle(RotationAngle),
    UpdateOutputMode(OutputMode),
}

#[component(pub)]
impl SimpleComponent for RotateBodyModel{
    type Init = ();

    type Input = ();
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
                            set_active: true
                        },

                        #[name(angle_combobox)]
                        DropDown {
                            set_hexpand: true,
                            set_model: Some(&StringList::new(&["90° clockwise", "180°", "90° counter-clockwise"])),

                            // TODO: Using container add for CellRendererText
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
                            set_group: Some(&default_angle_radiobutton)
                        },

                        #[name(angle_spinbutton)]
                        SpinButton {
                            // set_xalign: 1,
                            set_adjustment: &Adjustment::new(90f64, 1f64, 360f64, 1f64, 45f64, 45f64),
                            set_climb_rate: 1f64,
                            set_numeric: true
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
                            set_use_underline: true
                        },

                        #[name(name_entry)]
                        Entry {
                            set_text: ".rotated"
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
                        set_group: Some(&append_radiobutton)
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self>
    {
        let model = RotateBodyModel {
            custom_angle: 90,
            append: 90.to_string()
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}