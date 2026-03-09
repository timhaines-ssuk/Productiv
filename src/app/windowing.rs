use eframe::egui::{self, Pos2, Vec2, ViewportBuilder, ViewportCommand, ViewportId};
use tray::{MouseButton, MouseButtonState, TrayIconEvent};

use super::{ProductivApp, WIDGET_WIDTH, WidgetWindowState};

const ROOT_HIDE_POS: Pos2 = egui::pos2(-10_000.0, -10_000.0);
const WIDGET_MARGIN: f32 = 16.0;

pub(super) fn default_widget_window() -> WidgetWindowState {
    let (position, size) = default_widget_geometry();
    WidgetWindowState {
        visible: false,
        position,
        size,
        focus_pending: false,
    }
}

impl ProductivApp {
    pub(super) fn handle_root_close_requested(&mut self, ctx: &egui::Context) -> bool {
        if self.quit_requested {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return true;
        }

        if ctx.input(|input| input.viewport().close_requested()) {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.widget_window.visible = false;
        }

        false
    }

    pub(super) fn maintain_hidden_root(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(ROOT_HIDE_POS));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(1.0, 1.0)));
    }

    pub(super) fn process_tray_events(&mut self) {
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                if self.widget_window.visible {
                    self.widget_window.visible = false;
                } else {
                    self.widget_window = default_widget_window();
                    self.widget_window.visible = true;
                    self.widget_window.focus_pending = true;
                }
            }
        }
    }

    pub(super) fn show_widget_viewport(&mut self, ctx: &egui::Context) {
        let builder = ViewportBuilder::default()
            .with_title("Productiv")
            .with_inner_size(self.widget_window.size)
            .with_min_inner_size(self.widget_window.size)
            .with_max_inner_size(self.widget_window.size)
            .with_position(self.widget_window.position)
            .with_decorations(false)
            .with_resizable(false)
            .with_transparent(true)
            .with_taskbar(false)
            .with_always_on_top();

        ctx.show_viewport_immediate(widget_viewport_id(), builder, |widget_ctx, _class| {
            widget_ctx.request_repaint_after(std::time::Duration::from_secs(1));

            if self.widget_window.focus_pending {
                widget_ctx.send_viewport_cmd(ViewportCommand::Focus);
                self.widget_window.focus_pending = false;
            }

            if widget_ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
                self.widget_window.visible = false;
            }

            self.show_widget_contents(widget_ctx);

            if widget_ctx.input(|input| input.viewport().close_requested()) {
                self.widget_window.visible = false;
            }
        });
    }

    pub(super) fn hide_widget(&mut self) {
        self.widget_window.visible = false;
    }

    pub(super) fn request_quit(&mut self) {
        self.quit_requested = true;
    }
}

fn widget_viewport_id() -> ViewportId {
    ViewportId::from_hash_of("productiv-widget")
}

#[cfg(target_os = "windows")]
fn default_widget_geometry() -> (Pos2, Vec2) {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) } as f32;
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) } as f32;
    let widget_height = (screen_height - (WIDGET_MARGIN * 2.0)).min(960.0);
    let x = (screen_width - WIDGET_WIDTH - WIDGET_MARGIN).max(WIDGET_MARGIN);
    (
        egui::pos2(x, WIDGET_MARGIN),
        egui::vec2(WIDGET_WIDTH, widget_height),
    )
}

#[cfg(not(target_os = "windows"))]
fn default_widget_geometry() -> (Pos2, Vec2) {
    (egui::pos2(32.0, 32.0), egui::vec2(WIDGET_WIDTH, 840.0))
}
