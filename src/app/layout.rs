use chrono::{Days, Local};
use eframe::egui::{self, Align2, Color32, RichText};

use crate::app::planner::minutes_label;

use super::ProductivApp;

impl ProductivApp {
    pub(super) fn show_top_bar(&mut self, ctx: &egui::Context) {
        let tracker_status = self.runtime.status();
        let active_task_title = self
            .tasks
            .iter()
            .find(|task| Some(task.id) == self.runtime.active_task_id());
        let active_task_title = active_task_title.map(|task| task.title.clone());

        egui::TopBottomPanel::top("top_bar")
            .frame(egui::Frame::default().inner_margin(egui::Margin::same(12)))
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading("Productiv");
                    ui.label(
                        RichText::new(self.selected_day.format("%A, %d %b %Y").to_string())
                            .strong()
                            .color(Color32::from_rgb(198, 215, 247)),
                    );
                    ui.separator();
                    if ui.button("Today").clicked() {
                        self.selected_day = Local::now().date_naive();
                        self.refresh_all();
                    }
                    if ui.button("Previous").clicked() {
                        if let Some(day) = self.selected_day.checked_sub_days(Days::new(1)) {
                            self.selected_day = day;
                            self.refresh_all();
                        }
                    }
                    if ui.button("Next").clicked() {
                        if let Some(day) = self.selected_day.checked_add_days(Days::new(1)) {
                            self.selected_day = day;
                            self.refresh_all();
                        }
                    }
                    ui.separator();
                    ui.label("Default block");
                    ui.add(
                        egui::Slider::new(&mut self.default_plan_minutes, 30..=180)
                            .step_by(30.0)
                            .suffix(" min"),
                    );
                    if ui.button("Config").clicked() {
                        self.show_config_window = true;
                    }
                    ui.separator();
                    ui.label(format!(
                        "Tray: {}",
                        if self.tray_icon.is_some() {
                            "ready"
                        } else {
                            "unavailable"
                        }
                    ));
                    ui.label(format!("DB: {}", self.database.db_path().display()));
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "Tracker: {}",
                            if tracker_status.available {
                                "live"
                            } else {
                                "waiting"
                            }
                        ))
                        .color(if tracker_status.available {
                            Color32::from_rgb(126, 217, 87)
                        } else {
                            Color32::from_rgb(255, 209, 102)
                        }),
                    );
                    if let Some(task_title) = &active_task_title {
                        ui.separator();
                        ui.label(format!("Current task: {task_title}"));
                    }
                    if !tracker_status.window_title.is_empty() {
                        ui.separator();
                        ui.label(format!(
                            "{} | {}",
                            tracker_status.process_name, tracker_status.window_title
                        ));
                    }
                    if let Some(status) = &self.status_message {
                        ui.separator();
                        ui.label(RichText::new(status).color(Color32::from_rgb(255, 209, 102)));
                    }
                });
            });
    }

    pub(super) fn show_completion_prompt(&mut self, ctx: &egui::Context) {
        let Some(summary) = self.completion_prompt() else {
            return;
        };

        egui::Window::new("Task closed")
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(RichText::new(&summary.title).strong());
                ui.label(format!(
                    "Logged effort: {} ({})",
                    minutes_label(summary.minutes as i64),
                    summary.hours_label()
                ));
                if let Some(external_id) = &summary.external_id {
                    ui.label(format!(
                        "External item: #{external_id}. You can queue a writeback of the hours."
                    ));
                } else {
                    ui.label("This task is local-only, so the time stays in SQLite.");
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Keep local only").clicked() {
                        self.completion_prompt = None;
                    }
                    if ui
                        .add_enabled(
                            summary.external_id.is_some(),
                            egui::Button::new("Queue hours writeback"),
                        )
                        .clicked()
                    {
                        if let Err(error) = self
                            .database
                            .queue_remote_hours_writeback(summary.task_id, summary.minutes)
                        {
                            self.status_message =
                                Some(format!("Failed to queue hours writeback: {error}"));
                        } else {
                            self.status_message = Some(format!(
                                "Queued {} for remote writeback.",
                                summary.hours_label()
                            ));
                            self.refresh_all();
                        }
                        self.completion_prompt = None;
                    }
                });
            });
    }

    pub(super) fn show_config_window(&mut self, ctx: &egui::Context) {
        if !self.show_config_window {
            return;
        }

        let mut open = self.show_config_window;
        let mut save_clicked = false;
        let mut close_clicked = false;

        egui::Window::new("Configuration")
            .open(&mut open)
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.label("Azure DevOps");
                ui.add(
                    egui::TextEdit::singleline(&mut self.config_draft.azure_devops_org_url)
                        .hint_text("https://dev.azure.com/your-org"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.config_draft.azure_devops_project)
                        .hint_text("Project name"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.config_draft.azure_devops_pat)
                        .password(true)
                        .hint_text("PAT"),
                );
                ui.checkbox(
                    &mut self.config_draft.azure_devops_enabled,
                    "Enable Azure DevOps integration",
                );
                ui.add_space(10.0);

                ui.label("Runtime");
                ui.checkbox(&mut self.config_draft.outlook_enabled, "Enable Outlook sync");
                ui.checkbox(&mut self.config_draft.minimize_to_tray, "Minimize to tray");
                ui.horizontal(|ui| {
                    ui.label("Activity poll");
                    ui.add(
                        egui::DragValue::new(&mut self.config_draft.activity_poll_seconds)
                            .range(1..=30),
                    );
                    ui.label("sec");
                });
                ui.horizontal(|ui| {
                    ui.label("Idle threshold");
                    ui.add(
                        egui::DragValue::new(&mut self.config_draft.idle_threshold_minutes)
                            .range(1..=60),
                    );
                    ui.label("min");
                });

                ui.add_space(10.0);
                ui.label("The PAT currently lives in local SQLite under LocalAppData. A later pass can move secrets into Windows Credential Manager.");

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        save_clicked = true;
                    }
                    if ui.button("Close").clicked() {
                        close_clicked = true;
                    }
                });
            });

        self.show_config_window = open && !close_clicked;
        if save_clicked {
            self.save_config();
        }
    }
}
