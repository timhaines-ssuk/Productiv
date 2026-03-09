use std::sync::atomic::Ordering;

use eframe::egui::{self, Pos2, Vec2, ViewportCommand, WindowLevel};

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

    pub(super) fn process_tray_events(&mut self) {
        if self.tray_toggle_requested.swap(false, Ordering::Relaxed) {
            self.toggle_widget();
        }
    }

    pub(super) fn sync_root_widget_window(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));

        if self.widget_window.visible {
            let (position, size) = default_widget_geometry();
            self.widget_window.size = size;
            self.widget_window.position = position;
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(self.widget_window.size));
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(self.widget_window.position));
            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
            if self.widget_window.focus_pending {
                ctx.send_viewport_cmd(ViewportCommand::Focus);
                self.widget_window.focus_pending = false;
            }
            if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
                self.widget_window.visible = false;
            }
        } else {
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(ROOT_HIDE_POS));
        }
    }

    pub(super) fn hide_widget(&mut self) {
        self.widget_window.visible = false;
    }

    pub(super) fn request_quit(&mut self) {
        self.quit_requested = true;
    }

    fn toggle_widget(&mut self) {
        if self.widget_window.visible {
            self.widget_window.visible = false;
        } else {
            self.widget_window = default_widget_window();
            self.widget_window.visible = true;
            self.widget_window.focus_pending = true;
        }
    }
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
