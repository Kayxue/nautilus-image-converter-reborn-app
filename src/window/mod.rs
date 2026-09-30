use gtk::{
    Box, Button, HeaderBar, Orientation, Widget, Window,
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
        resizer::{ResizeKind, ResizerConfig},
        rotator::{RotationAngle, RotationAngleKind::Ninety, RotatorConfig},
    },
    window::window_body::{ResizeBodyModel, ResizeBodyOutput, RotateBodyModel, RotateBodyOutput},
};

mod window_body;

pub struct Initializer {
    pub mode: Mode,
    pub paths: Vec<String>,
    /// XID of the parent (Nautilus) window passed by the extension (X11 only).
    pub parent_xid: Option<u64>,
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

/// Keeps the body component controller alive for the lifetime of the window.
/// If this is dropped the output channel closes and signal-handler `.unwrap()`s panic.
enum BodyController {
    Resize(Controller<ResizeBodyModel>),
    Rotate(Controller<RotateBodyModel>),
}

pub struct AppModel {
    general_config: GeneralConfig,
    header: Controller<HeaderModel>,
    body_widget: Widget,
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
                    body_widget: widget.clone().into(),
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
                        rotation_angle: Some(RotationAngle::Specific(Ninety)),
                        image_size: None,
                        output_mode: OutputMode::NewFile(".rotated".to_owned()),
                    },
                    header,
                    body_widget: widget.clone().into(),
                    _body: BodyController::Rotate(rotate_body),
                };
                (widget, model)
            }
            Mode::Convert => todo!(),
        };

        let widgets = view_output!();
        widgets.dialog_vbox1.append(&body_widget);

        // ── Top-window detection ──────────────────────────────────────────────
        // If the Nautilus extension passed its window XID, bind our window as a
        // transient child so the WM keeps it stacked above Nautilus.
        // We deliberately do NOT set `destroy-with-parent`; if Nautilus closes
        // while we are still running, we just clear the transient hint so our
        // window survives as an independent top-level.
        if let Some(parent_xid) = init.parent_xid {
            bind_transient_to_parent(&root, parent_xid);
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInput::Cancel => {
                main_application().quit();
            }
            AppInput::Execute => {
                //TODO: Implement actual logic
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

// ── Parent-window binding helpers ─────────────────────────────────────────────

/// Binds `window` as a transient child of the X11 window identified by
/// `parent_xid`.
///
/// The actual `XSetTransientForHint` call is deferred to `connect_realize` so
/// the window's own X11 surface (and its XID) is guaranteed to exist.
///
/// If GDK already tracks the parent surface (same-process scenario) we also
/// watch it via `connect_mapped_notify`: when the parent is unmapped/destroyed
/// we clear the transient hint instead of closing, keeping our window alive.
///
/// On a pure-Wayland session (no XWayland) the display will not be an
/// `X11Display` and this function is a no-op.
fn bind_transient_to_parent(window: &Window, parent_xid: u64) {
    use gdk4_x11::{X11Display, X11Surface, prelude::*};
    use gtk::prelude::NativeExt;

    // Defer to connect_realize: the window's GDK surface only exists after the
    // underlying OS window is created.
    window.connect_realize(move |win| {
        // Verify the display backend is X11 / XWayland.
        let display = match win.display().downcast::<X11Display>() {
            Ok(d) => d,
            Err(_) => {
                eprintln!(
                    "[nautilus-image-converter] Pure Wayland session detected; \
                     skipping X11 parent-window binding."
                );
                return;
            }
        };

        // Obtain our window's X11 surface and its XID.
        let our_surface = match win.surface().and_then(|s| s.downcast::<X11Surface>().ok()) {
            Some(s) => s,
            None => {
                eprintln!(
                    "[nautilus-image-converter] Could not downcast window surface \
                     to X11Surface."
                );
                return;
            }
        };
        let our_xid = our_surface.xid();

        // Set WM_TRANSIENT_FOR so the window manager stacks us above Nautilus.
        // SAFETY: `display.xdisplay()` is a valid *mut xlib::Display from GDK.
        unsafe { set_transient_for_xlib(&display, our_xid as u64, parent_xid) };

        // If GDK already tracks the parent surface in this process (uncommon in
        // cross-process use, but possible if the extension is ever in-process),
        // watch it for unmapping.  When it becomes unmapped we clear the
        // WM_TRANSIENT_FOR hint so the WM treats our window as a standalone
        // top-level and does not close/hide it along with the parent.
        if let Some(parent_surf) = X11Surface::lookup_for_display(&display, parent_xid) {
            let our_surf_clone = our_surface.clone();
            let display_clone = display.clone();
            parent_surf.connect_mapped_notify(move |surf| {
                if !surf.is_mapped() {
                    // Parent window has gone away — decouple our window.
                    unsafe {
                        clear_transient_for_xlib(
                            &display_clone,
                            our_surf_clone.xid() as u64,
                            display_clone.xrootwindow() as u64,
                        )
                    };
                }
            });
        }
    });
}

// Minimal FFI binding to the one Xlib function we need.
// GDK4 already links against libX11, so this is always available on X11/XWayland.
#[link(name = "X11")]
unsafe extern "C" {
    fn XSetTransientForHint(
        display: *mut std::ffi::c_void,
        w: std::os::raw::c_ulong,
        prop_window: std::os::raw::c_ulong,
    ) -> std::os::raw::c_int;
}

/// Sets `WM_TRANSIENT_FOR` on `our_xid` pointing at `parent_xid` via Xlib.
///
/// # Safety
/// `display` must be a live GDK X11 display; both XIDs must be valid X11
/// window identifiers.
unsafe fn set_transient_for_xlib(display: &gdk4_x11::X11Display, our_xid: u64, parent_xid: u64) {
    let xdisplay = unsafe { display.xdisplay() } as *mut std::ffi::c_void;
    unsafe {
        XSetTransientForHint(
            xdisplay,
            our_xid as std::os::raw::c_ulong,
            parent_xid as std::os::raw::c_ulong,
        );
    }
}

/// Clears the `WM_TRANSIENT_FOR` hint by pointing it at `root_xid` (the
/// ICCCM §4.1.2.6 way to say "no parent").
///
/// # Safety
/// Same requirements as [`set_transient_for_xlib`].
unsafe fn clear_transient_for_xlib(display: &gdk4_x11::X11Display, our_xid: u64, root_xid: u64) {
    let xdisplay = unsafe { display.xdisplay() } as *mut std::ffi::c_void;
    unsafe {
        XSetTransientForHint(
            xdisplay,
            our_xid as std::os::raw::c_ulong,
            root_xid as std::os::raw::c_ulong,
        );
    }
}
