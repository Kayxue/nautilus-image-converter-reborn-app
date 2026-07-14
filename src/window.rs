use gtk::{
    Adjustment, Align, Box, Button, CheckButton, ComboBoxText, DropDown, HeaderBar, Label,
    Orientation, SpinButton, StringList, Window,
    prelude::{BoxExt, ButtonExt, CheckButtonExt, GtkWindowExt, OrientableExt, WidgetExt},
};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, component,
};

use crate::{
    Mode, OutputMode,
    manipulators::{
        resizer::{ResizeKind, ResizerConfig},
        rotator::{RotationAngle, RotationAngleKind::Ninety, RotatorConfig},
    },
};

pub struct Initializer {
    pub mode: Mode,
    pub paths: Vec<String>,
}

pub struct GeneralConfig {
    pub mode: Mode,
    pub rotation_angle: Option<RotationAngle>,
    pub image_size: Option<ResizeKind>,
    pub output_mode: OutputMode,
}

impl Into<RotatorConfig> for GeneralConfig {
    fn into(self) -> RotatorConfig {
        RotatorConfig(self.rotation_angle.unwrap())
    }
}

impl Into<ResizerConfig> for GeneralConfig {
    fn into(self) -> ResizerConfig {
        ResizerConfig(self.image_size.unwrap_or(ResizeKind::Custom(0, 0)))
    }
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
            pack_start = &Button {
                set_label: "Cancel",
                connect_clicked[sender] => move |_|{
                    sender.output(HeaderOutput::Cancel).unwrap()
                },
            },
            pack_end = &Button {
                set_label: &format!("{}",model.mode),
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

pub struct ResizeBodyModel {
    pub cur_percent: u8,
    pub cur_width: u16,
    pub cur_height: u16,
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
        Box {
            set_hexpand: true,
            set_orientation: Orientation::Vertical,
            set_margin_all: 12,
            set_spacing: 12,

            Label {
                set_valign: Align::Center,
                set_xalign: 0f32,
                set_label: "&lt;b&gt;Image Size&lt;/b&gt;",
                set_use_markup: true
            },

            Box {
                set_valign: Align::Center,
                set_spacing: 12,

                Label {
                    set_halign: Align::Center
                },

                Box {
                    set_hexpand: true,
                    set_orientation: Orientation::Vertical,
                    set_spacing: 6,

                    Box {
                        set_valign: Align::Center,
                        set_spacing: 6,

                        #[name(default_size_radiobutton)]
                        CheckButton {
                            set_halign: Align::Center,
                            set_label: Some("Select a size"),
                            set_use_underline: true
                        },

                        DropDown {
                            set_hexpand: true,
                            set_model: Some(&StringList::new(&["96x96", "128x128", "640x640", "800x800", "1024x768", "1280x960"]))
                        },

                        Label {
                            set_halign: Align::Center,
                            set_label: "pixels"
                        }
                    }
                },

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

                    SpinButton {
                        set_hexpand: true,
                        set_adjustment: &Adjustment::new(50f64, 1f64, 100f64, 1f64, 10f64, 0f64)
                    },

                    Label {
                        set_halign: Align::Center,
                        set_label: "percent"
                    }
                },

                #[name(hbox5)]
                Box {

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
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

pub struct RotateBodyModel {}

pub enum RotateBodyOutput {
    UpdateAngle(RotationAngle),
    UpdateOutputMode(OutputMode),
}

pub struct AppModel {
    general_config: GeneralConfig,
    header: Controller<HeaderModel>,
}

#[derive(Debug)]
pub enum AppInput {
    Cancel,
    Execute,
    UpdateImageSize(ResizeKind),
    UpdateAngle(RotationAngle),
    UpdateOutputMode(OutputMode),
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
            set_titlebar: Some(model.header.widget())
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
        let model = match init.mode {
            Mode::Resize => AppModel {
                general_config: GeneralConfig {
                    mode: Mode::Resize,
                    rotation_angle: None,
                    image_size: Some(ResizeKind::Percentage(0.5)),
                    output_mode: OutputMode::NewFile(".resized".to_owned()),
                },
                header,
            },
            Mode::Rotate => AppModel {
                general_config: GeneralConfig {
                    mode: Mode::Rotate,
                    rotation_angle: Some(RotationAngle::Specific(Ninety)),
                    image_size: None,
                    output_mode: OutputMode::NewFile(".rotated".to_owned()),
                },
                header,
            },
            Mode::Convert => todo!(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
