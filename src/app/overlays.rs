use eframe::egui::{self, Align2, Color32, RichText};

use crate::app::timeline::minutes_label;

use super::ProductivApp;

impl ProductivApp {
    pub(super) fn show_completion_prompt(&mut self, ctx: &egui::Context) {
        let Some(summary) = self.completion_prompt.clone() else {
            return;
        };

        egui::Window::new("Task closed")
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.label(RichText::new(&summary.title).strong());
                ui.label(format!(
                    "Logged effort: {} ({})",
                    minutes_label(summary.minutes as i64),
                    summary.hours_label()
                ));

                if let Some(external_id) = &summary.external_id {
                    ui.label(
                        RichText::new(format!(
                            "Azure DevOps item #{external_id}. Queue the hours writeback?"
                        ))
                        .color(Color32::from_rgb(107, 116, 126)),
                    );
                } else {
                    ui.label(
                        RichText::new("This task is local-only, so the time stays in SQLite.")
                            .color(Color32::from_rgb(107, 116, 126)),
                    );
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Keep local only").clicked() {
                        self.completion_prompt = None;
                    }
                    if ui
                        .add_enabled(
                            summary.external_id.is_some(),
                            egui::Button::new("Queue writeback"),
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
                                "Queued {} for Azure DevOps writeback.",
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

        egui::Window::new("Preferences")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.label(RichText::new("Azure DevOps").strong());
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
                ui.label(RichText::new("Runtime").strong());
                ui.checkbox(&mut self.config_draft.outlook_enabled, "Enable Outlook sync");
                ui.checkbox(&mut self.config_draft.minimize_to_tray, "Use tray widget");
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
                ui.label(
                    RichText::new(
                        "The PAT is currently stored in local SQLite under LocalAppData. Moving secrets to Windows Credential Manager is still pending.",
                    )
                    .small()
                    .color(Color32::from_rgb(107, 116, 126)),
                );
                ui.label(
                    RichText::new(format!(
                        "Database path: {}",
                        self.database.db_path().display()
                    ))
                    .small()
                    .color(Color32::from_rgb(107, 116, 126)),
                );

                ui.add_space(10.0);
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
