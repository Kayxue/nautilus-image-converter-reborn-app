use gtk::{
    Box, Button, HeaderBar, Orientation, Widget, Window,
    prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt},
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
    window::window_body::{ResizeBodyModel, ResizeBodyOutput, RotateBodyModel, RotateBodyOutput},
};

mod window_body;

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

pub struct AppModel {
    general_config: GeneralConfig,
    header: Controller<HeaderModel>,
    body_widget: Widget,
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
            set_titlebar: Some(model.header.widget()),

            #[name(dialog_vbox1)]
            Box {
                set_orientation: Orientation::Vertical,
                set_spacing: 6,
                set_margin_all: 12,
                set_hexpand: true,
                set_vexpand: true,
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
                    body_widget: widget.into(),
                };
                (resize_body.widget().clone(), model)
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
                        rotation_angle: Some(RotationAngle::Specific(Ninety)),
                        image_size: None,
                        output_mode: OutputMode::NewFile(".rotated".to_owned()),
                    },
                    header,
                    body_widget: widget.into(),
                };
                (rotate_body.widget().clone(), model)
            }
            Mode::Convert => todo!(),
        };

        let widgets = view_output!();
        widgets.dialog_vbox1.append(&body_widget);

        ComponentParts { model, widgets }
    }
}
