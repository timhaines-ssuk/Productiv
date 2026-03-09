mod actions;
mod cards;
mod overlays;
mod timeline;
mod windowing;

use std::time::{Duration, Instant};

use chrono::{Local, NaiveDate};
use eframe::egui::{self, Color32, Pos2, Vec2};
use tray::{Icon, TrayIcon, TrayIconBuilder};

use crate::{
    models::{ActivitySegment, AppConfig, CalendarEvent, ScheduleBlock, Task, TaskCompletionDraft},
    services::BackgroundRuntime,
    storage::Database,
};

pub(super) const DAY_START_MINUTE: i32 = 7 * 60;
pub(super) const DAY_END_MINUTE: i32 = 20 * 60;
pub(super) const SLOT_MINUTES: i32 = 30;
pub(super) const WIDGET_WIDTH: f32 = 500.0;

#[derive(Clone, Copy, Debug)]
pub(super) enum DragPayload {
    Task(i64),
    Block(i64),
}

#[derive(Clone, Copy, Debug)]
pub(super) struct WidgetWindowState {
    pub visible: bool,
    pub position: Pos2,
    pub size: Vec2,
    pub focus_pending: bool,
}

pub struct ProductivApp {
    pub(super) database: Database,
    pub(super) runtime: BackgroundRuntime,
    pub(super) _tray_icon: Option<TrayIcon>,
    pub(super) selected_day: NaiveDate,
    pub(super) default_plan_minutes: i32,
    pub(super) draft_task_title: String,
    pub(super) draft_task_description: String,
    pub(super) draft_task_estimate_hours: f32,
    pub(super) config_draft: AppConfig,
    pub(super) show_config_window: bool,
    pub(super) tasks: Vec<Task>,
    pub(super) schedule_blocks: Vec<ScheduleBlock>,
    pub(super) calendar_events: Vec<CalendarEvent>,
    pub(super) recent_activity: Vec<ActivitySegment>,
    pub(super) completion_prompt: Option<TaskCompletionDraft>,
    pub(super) status_message: Option<String>,
    pub(super) last_refresh: Instant,
    pub(super) widget_window: WidgetWindowState,
    pub(super) quit_requested: bool,
}

impl ProductivApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        database: Database,
        runtime: BackgroundRuntime,
    ) -> Self {
        configure_visuals(&cc.egui_ctx);

        let config_draft = database.load_app_config().unwrap_or_default();
        let mut app = Self {
            database,
            runtime,
            _tray_icon: create_tray_icon().ok(),
            selected_day: Local::now().date_naive(),
            default_plan_minutes: 60,
            draft_task_title: String::new(),
            draft_task_description: String::new(),
            draft_task_estimate_hours: 1.0,
            config_draft,
            show_config_window: false,
            tasks: Vec::new(),
            schedule_blocks: Vec::new(),
            calendar_events: Vec::new(),
            recent_activity: Vec::new(),
            completion_prompt: None,
            status_message: None,
            last_refresh: Instant::now() - Duration::from_secs(60),
            widget_window: windowing::default_widget_window(),
            quit_requested: false,
        };
        app.refresh_all();
        app
    }
}

impl eframe::App for ProductivApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.handle_root_close_requested(ctx) {
            return;
        }

        if self.last_refresh.elapsed() > Duration::from_secs(3) {
            self.refresh_all();
        }

        self.maintain_hidden_root(ctx);
        self.process_tray_events();

        if self.widget_window.visible {
            self.show_widget_viewport(ctx);
        }

        ctx.request_repaint_after(Duration::from_millis(100));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}

fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = Color32::from_rgba_unmultiplied(0, 0, 0, 0);
    visuals.window_fill = Color32::from_rgb(248, 245, 239);
    visuals.extreme_bg_color = Color32::from_rgb(234, 229, 222);
    visuals.override_text_color = Some(Color32::from_rgb(38, 44, 56));
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(241, 237, 231);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(235, 230, 223);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(225, 236, 232);
    visuals.widgets.active.bg_fill = Color32::from_rgb(202, 220, 213);
    visuals.widgets.noninteractive.bg_stroke.color = Color32::from_rgb(216, 210, 202);
    visuals.window_shadow.color = Color32::from_black_alpha(30);
    ctx.set_visuals(visuals);
}

fn create_tray_icon() -> anyhow::Result<TrayIcon> {
    let icon = Icon::from_rgba(productiv_icon_rgba(), 32, 32)?;
    TrayIconBuilder::new()
        .with_tooltip("Productiv")
        .with_icon(icon)
        .build()
        .map_err(anyhow::Error::from)
}

fn productiv_icon_rgba() -> Vec<u8> {
    let mut rgba = vec![0u8; 32 * 32 * 4];
    for y in 0..32 {
        for x in 0..32 {
            let idx = ((y * 32 + x) * 4) as usize;
            let edge = x < 2 || x > 29 || y < 2 || y > 29;
            let lane = (8..24).contains(&x) && (6..28).contains(&y);
            let pulse = (12..20).contains(&x) && (10..24).contains(&y);
            let color = if edge {
                [214, 138, 67, 255]
            } else if pulse {
                [104, 168, 138, 255]
            } else if lane {
                [73, 118, 108, 255]
            } else {
                [245, 240, 232, 255]
            };
            rgba[idx..idx + 4].copy_from_slice(&color);
        }
    }
    rgba
}
