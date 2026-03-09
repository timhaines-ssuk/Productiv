mod actions;
mod layout;
mod planner;
mod sidebar;

use std::time::{Duration, Instant};

use chrono::{Local, NaiveDate};
use eframe::egui;
use tray::{Icon, TrayIcon, TrayIconBuilder};

use crate::{
    models::{ActivitySegment, AppConfig, CalendarEvent, ScheduleBlock, Task, TaskCompletionDraft},
    services::BackgroundRuntime,
    storage::Database,
};

pub(super) const DAY_START_MINUTE: i32 = 7 * 60;
pub(super) const DAY_END_MINUTE: i32 = 20 * 60;
pub(super) const SLOT_MINUTES: i32 = 30;
pub(super) const SLOT_HEIGHT: f32 = 54.0;

#[derive(Clone, Copy, Debug)]
pub(super) enum DragPayload {
    Task(i64),
    Block(i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SelectedItem {
    Meeting(i64),
    Block(i64),
}

pub struct ProductivApp {
    pub(super) database: Database,
    pub(super) runtime: BackgroundRuntime,
    pub(super) tray_icon: Option<TrayIcon>,
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
    pub(super) selected_item: Option<SelectedItem>,
    pub(super) completion_prompt: Option<TaskCompletionDraft>,
    pub(super) status_message: Option<String>,
    pub(super) last_refresh: Instant,
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
            tray_icon: create_tray_icon().ok(),
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
            selected_item: None,
            completion_prompt: None,
            status_message: None,
            last_refresh: Instant::now() - Duration::from_secs(60),
        };
        app.refresh_all();
        app
    }
}

impl eframe::App for ProductivApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(1));
        if self.last_refresh.elapsed() > Duration::from_secs(3) {
            self.refresh_all();
        }

        self.show_completion_prompt(ctx);
        self.show_config_window(ctx);
        self.show_top_bar(ctx);
        self.show_task_panel(ctx);
        self.show_detail_panel(ctx);
        self.show_planner(ctx);
    }
}

fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(11, 16, 24);
    visuals.window_fill = egui::Color32::from_rgb(16, 22, 31);
    visuals.override_text_color = Some(egui::Color32::from_rgb(230, 235, 240));
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(18, 25, 34);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(44, 81, 118);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(42, 64, 89);
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
                [221, 145, 65, 255]
            } else if pulse {
                [96, 191, 130, 255]
            } else if lane {
                [47, 120, 108, 255]
            } else {
                [20, 27, 38, 255]
            };
            rgba[idx..idx + 4].copy_from_slice(&color);
        }
    }
    rgba
}
