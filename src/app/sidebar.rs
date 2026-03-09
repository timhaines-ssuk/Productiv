use chrono::Local;
use eframe::egui::{self, Color32, RichText, Sense};

use crate::{
    app::planner::{minute_label, minutes_label},
    models::TaskState,
};

use super::{DragPayload, ProductivApp};

impl ProductivApp {
    pub(super) fn show_task_panel(&mut self, ctx: &egui::Context) {
        let active_task_id = self.runtime.active_task_id();
        let tasks = self.tasks.clone();

        egui::SidePanel::left("tasks")
            .resizable(true)
            .default_width(350.0)
            .show(ctx, |ui| {
                ui.heading("Task List");
                ui.label("Draft local tasks, then drag them into the day itinerary.");
                ui.add_space(8.0);

                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(24, 31, 43))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Draft task").strong());
                        ui.add(
                            egui::TextEdit::singleline(&mut self.draft_task_title)
                                .hint_text("Task title"),
                        );
                        ui.add(
                            egui::TextEdit::multiline(&mut self.draft_task_description)
                                .desired_rows(3)
                                .hint_text("Optional notes, acceptance criteria, context"),
                        );
                        ui.horizontal(|ui| {
                            ui.label("Estimate");
                            ui.add(
                                egui::DragValue::new(&mut self.draft_task_estimate_hours)
                                    .range(0.5..=16.0)
                                    .speed(0.25)
                                    .suffix(" h"),
                            );
                            if ui.button("Add task").clicked() {
                                self.create_task();
                            }
                        });
                    });

                ui.add_space(12.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for task in tasks {
                        let is_active = Some(task.id) == active_task_id;
                        let card_fill = match task.state {
                            TaskState::Done => Color32::from_rgb(28, 46, 34),
                            TaskState::Active => Color32::from_rgb(32, 50, 71),
                            _ => Color32::from_rgb(18, 24, 34),
                        };

                        egui::Frame::group(ui.style())
                            .fill(card_fill)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    let title = if let Some(external_id) = &task.external_id {
                                        format!("#{external_id} {}", task.title)
                                    } else {
                                        task.title.clone()
                                    };
                                    let response = ui.add(
                                        egui::Label::new(RichText::new(title).strong())
                                            .sense(Sense::click_and_drag()),
                                    );
                                    if response.drag_started() {
                                        response.dnd_set_drag_payload(DragPayload::Task(task.id));
                                    }
                                    if is_active {
                                        ui.label(
                                            RichText::new("Tracking")
                                                .color(Color32::from_rgb(126, 217, 87))
                                                .strong(),
                                        );
                                    } else {
                                        ui.label(task.state.as_str());
                                    }
                                });

                                if !task.description.is_empty() {
                                    ui.label(RichText::new(&task.description).small());
                                }

                                ui.horizontal_wrapped(|ui| {
                                    if ui
                                        .add_enabled(
                                            !is_active && task.state != TaskState::Done,
                                            egui::Button::new("Start"),
                                        )
                                        .clicked()
                                    {
                                        self.set_active_task(Some(task.id));
                                    }
                                    if ui
                                        .add_enabled(is_active, egui::Button::new("Stop"))
                                        .clicked()
                                    {
                                        self.set_active_task(None);
                                    }
                                    if ui
                                        .add_enabled(
                                            task.state != TaskState::Done,
                                            egui::Button::new("Done"),
                                        )
                                        .clicked()
                                    {
                                        self.complete_task(task.id);
                                    }
                                });

                                ui.horizontal_wrapped(|ui| {
                                    ui.label(format!(
                                        "Tracked: {}",
                                        minutes_label(task.logged_minutes as i64)
                                    ));
                                    if let Some(estimate) = task.estimated_minutes {
                                        ui.separator();
                                        ui.label(format!(
                                            "Estimate: {}",
                                            minutes_label(estimate as i64)
                                        ));
                                    }
                                    if let Some(pending) = task.pending_remote_writeback_minutes {
                                        ui.separator();
                                        ui.label(format!(
                                            "Writeback queued: {:.2}h",
                                            pending as f32 / 60.0
                                        ));
                                    }
                                });
                            });
                        ui.add_space(8.0);
                    }
                });
            });
    }

    pub(super) fn show_detail_panel(&mut self, ctx: &egui::Context) {
        let tracker_status = self.runtime.status();
        let recent_activity = self.recent_activity.clone();

        egui::SidePanel::right("details")
            .resizable(true)
            .default_width(330.0)
            .show(ctx, |ui| {
                ui.heading("Inspector");

                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(21, 27, 37))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Tracker status").strong());
                        ui.label(tracker_status.tracking_note);
                        ui.label(format!(
                            "Foreground process: {}",
                            tracker_status.process_name
                        ));
                        ui.label(format!("Window: {}", tracker_status.window_title));
                        ui.label(format!("Class: {}", tracker_status.window_class));
                        ui.label(format!("Idle: {}s", tracker_status.idle_seconds));
                        if let Some(task_id) = tracker_status.current_task_id {
                            ui.label(format!("Tracked task id: {task_id}"));
                        }
                        ui.label(format!(
                            "Last sample: {}",
                            tracker_status
                                .last_sample_at
                                .with_timezone(&Local)
                                .format("%H:%M:%S")
                        ));
                    });

                ui.add_space(10.0);

                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(21, 27, 37))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Selected itinerary item").strong());
                        if let Some(block) = self.selected_block() {
                            ui.label(RichText::new(&block.title).strong());
                            ui.label(format!(
                                "{} - {}",
                                minute_label(block.start_minute),
                                minute_label(block.end_minute)
                            ));
                            if let Some(task_id) = block.task_id {
                                if ui.button("Start linked task").clicked() {
                                    self.set_active_task(Some(task_id));
                                }
                            }
                            if ui.button("Remove block").clicked() {
                                if let Err(error) = self.database.delete_schedule_block(block.id) {
                                    self.status_message =
                                        Some(format!("Failed to delete block: {error}"));
                                } else {
                                    self.selected_item = None;
                                    self.refresh_all();
                                }
                            }
                        } else if let Some(meeting) = self.selected_meeting() {
                            ui.label(RichText::new(&meeting.title).strong());
                            ui.label(format!(
                                "{} - {}",
                                minute_label(meeting.start_minute),
                                minute_label(meeting.end_minute)
                            ));
                            if !meeting.location.is_empty() {
                                ui.label(format!("Location: {}", meeting.location));
                            }
                            if !meeting.organizer.is_empty() {
                                ui.label(format!("Organizer: {}", meeting.organizer));
                            }
                            if !meeting.body.is_empty() {
                                ui.separator();
                                ui.label(meeting.body);
                            }
                        } else {
                            ui.label("Click a scheduled block or meeting to inspect it.");
                        }
                    });

                ui.add_space(10.0);

                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(21, 27, 37))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Integration status").strong());
                        ui.label(format!(
                            "Outlook: {}",
                            if self.config_draft.outlook_enabled {
                                "enabled in config, sync stubbed"
                            } else {
                                "disabled"
                            }
                        ));
                        ui.label(format!(
                            "Azure DevOps: {}",
                            if self.config_draft.azure_devops_enabled {
                                "enabled in config, sync stubbed"
                            } else {
                                "disabled"
                            }
                        ));
                        ui.label("SQLite: stored in LocalAppData under Productiv.");
                    });

                ui.add_space(10.0);

                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(21, 27, 37))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Recent activity").strong());
                        for segment in recent_activity {
                            ui.separator();
                            ui.label(format!(
                                "{} - {}",
                                segment.started_at.with_timezone(&Local).format("%H:%M"),
                                segment.ended_at.with_timezone(&Local).format("%H:%M")
                            ));
                            ui.label(format!(
                                "{} | {}",
                                segment.process_name, segment.window_title
                            ));
                            if let Some(task_id) = segment.task_id {
                                ui.label(format!("Linked task id: {task_id}"));
                            }
                        }
                    });
            });
    }
}
