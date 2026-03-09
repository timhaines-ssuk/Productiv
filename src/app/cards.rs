use chrono::{Days, Local};
use eframe::egui::{self, Align, Color32, CornerRadius, Id, Margin, RichText, Stroke};

use crate::models::{Task, TaskState};

use super::{DragPayload, ProductivApp};
use crate::app::timeline::{minute_label, minutes_label};

impl ProductivApp {
    pub(super) fn show_widget_contents(&mut self, ctx: &egui::Context) {
        self.show_completion_prompt(ctx);
        self.show_config_window(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                widget_shell_frame().show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            self.show_header_card(ui);
                            self.show_overview_card(ui);
                            self.show_tasks_card(ui);
                            self.show_day_plan_card(ui);
                            self.show_activity_card(ui);

                            if let Some(message) = &self.status_message {
                                section_frame().show(ui, |ui| {
                                    ui.label(
                                        RichText::new(message)
                                            .color(Color32::from_rgb(145, 88, 36))
                                            .strong(),
                                    );
                                });
                            }
                        });
                });
            });
    }

    fn show_header_card(&mut self, ui: &mut egui::Ui) {
        let tracker = self.runtime.status();
        let open_tasks = self
            .tasks
            .iter()
            .filter(|task| task.state != TaskState::Done)
            .count();
        let planned_blocks = self
            .schedule_blocks
            .iter()
            .filter(|block| block.task_id.is_some())
            .count();
        let queued_minutes: i32 = self
            .tasks
            .iter()
            .filter_map(|task| task.pending_remote_writeback_minutes)
            .sum();

        section_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Productiv")
                            .size(22.0)
                            .strong()
                            .color(Color32::from_rgb(44, 63, 71)),
                    );
                    ui.label(
                        RichText::new(self.selected_day.format("%A, %d %B").to_string())
                            .color(Color32::from_rgb(103, 111, 120)),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("Quit").clicked() {
                        self.request_quit();
                    }
                    if ui.small_button("Hide").clicked() {
                        self.hide_widget();
                    }
                    if ui.small_button("Prefs").clicked() {
                        self.show_config_window = true;
                    }
                });
            });

            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                if ui.small_button("Today").clicked() {
                    self.selected_day = Local::now().date_naive();
                    self.refresh_all();
                }
                if ui.small_button("Previous").clicked() {
                    if let Some(day) = self.selected_day.checked_sub_days(Days::new(1)) {
                        self.selected_day = day;
                        self.refresh_all();
                    }
                }
                if ui.small_button("Next").clicked() {
                    if let Some(day) = self.selected_day.checked_add_days(Days::new(1)) {
                        self.selected_day = day;
                        self.refresh_all();
                    }
                }

                ui.separator();
                chip(
                    ui,
                    if tracker.available {
                        "Tracker live"
                    } else {
                        "Tracker waiting"
                    },
                    if tracker.available {
                        Color32::from_rgb(92, 149, 123)
                    } else {
                        Color32::from_rgb(190, 145, 84)
                    },
                );
                chip(
                    ui,
                    &format!("{open_tasks} open tasks"),
                    Color32::from_rgb(102, 126, 146),
                );
                chip(
                    ui,
                    &format!("{planned_blocks} planned blocks"),
                    Color32::from_rgb(77, 134, 118),
                );
                if queued_minutes > 0 {
                    chip(
                        ui,
                        &format!("{} queued", minutes_label(queued_minutes as i64)),
                        Color32::from_rgb(167, 112, 63),
                    );
                }
            });
        });
    }

    fn show_overview_card(&mut self, ui: &mut egui::Ui) {
        let tracker = self.runtime.status();
        let active_task = self
            .tasks
            .iter()
            .find(|task| Some(task.id) == self.runtime.active_task_id())
            .map(|task| task.title.clone());
        let current_meeting = self.current_meeting();
        let next_meeting = self.next_meeting();

        section_frame().show(ui, |ui| {
            section_heading(
                ui,
                "Overview",
                "Current context without the dashboard noise.",
            );

            mini_card(ui, "Focus", |ui| {
                if let Some(task_title) = active_task {
                    ui.label(RichText::new(task_title).strong());
                } else {
                    ui.label("No active task");
                }
                if !tracker.window_title.is_empty() {
                    ui.label(
                        RichText::new(format!(
                            "{} | {}",
                            tracker.process_name, tracker.window_title
                        ))
                        .small()
                        .color(Color32::from_rgb(106, 115, 126)),
                    );
                }
            });

            ui.add_space(8.0);

            mini_card(ui, "Meetings", |ui| {
                if let Some(meeting) = current_meeting {
                    ui.label(RichText::new("In meeting").strong());
                    ui.label(format!(
                        "{} · ends {}",
                        meeting.title,
                        minute_label(meeting.end_minute)
                    ));
                } else if let Some(meeting) = next_meeting {
                    ui.label(RichText::new("Next meeting").strong());
                    ui.label(format!(
                        "{} · {}",
                        meeting.title,
                        minute_label(meeting.start_minute)
                    ));
                } else {
                    ui.label("No more meetings on this day");
                }
            });

            ui.add_space(8.0);

            mini_card(ui, "Tracker", |ui| {
                ui.label(tracker.tracking_note);
                ui.label(
                    RichText::new(format!("Idle {}s", tracker.idle_seconds))
                        .small()
                        .color(Color32::from_rgb(106, 115, 126)),
                );
            });
        });
    }

    fn show_tasks_card(&mut self, ui: &mut egui::Ui) {
        let visible_tasks: Vec<Task> = self
            .tasks
            .iter()
            .filter(|task| task.state != TaskState::Done)
            .cloned()
            .collect();
        let active_task_id = self.runtime.active_task_id();

        section_frame().show(ui, |ui| {
            section_heading(
                ui,
                "Task Inbox",
                "Draft tasks here, then drag them into the day plan.",
            );

            egui::Frame::new()
                .fill(Color32::from_rgb(239, 235, 228))
                .corner_radius(CornerRadius::same(14))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.draft_task_title)
                            .hint_text("Task title"),
                    );
                    ui.add(
                        egui::TextEdit::multiline(&mut self.draft_task_description)
                            .desired_rows(2)
                            .hint_text("Optional notes"),
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

            ui.add_space(10.0);

            egui::ScrollArea::vertical()
                .max_height(230.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for task in visible_tasks {
                        let is_active = Some(task.id) == active_task_id;
                        ui.dnd_drag_source(
                            Id::new(("task", task.id)),
                            DragPayload::Task(task.id),
                            |ui| {
                                task_tile_frame(task_tint(&task)).show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            let title = task_label(&task);
                                            ui.label(RichText::new(title).strong());
                                            ui.horizontal_wrapped(|ui| {
                                                chip(ui, task.state.as_str(), task_tint(&task));
                                                if let Some(estimate) = task.estimated_minutes {
                                                    chip(
                                                        ui,
                                                        &minutes_label(estimate as i64),
                                                        Color32::from_rgb(122, 133, 147),
                                                    );
                                                }
                                            });
                                        });
                                        ui.with_layout(
                                            egui::Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if ui
                                                    .add_enabled(
                                                        task.state != TaskState::Done,
                                                        egui::Button::new("Done"),
                                                    )
                                                    .clicked()
                                                {
                                                    self.complete_task(task.id);
                                                }
                                                if is_active {
                                                    if ui.button("Stop").clicked() {
                                                        self.set_active_task(None);
                                                    }
                                                } else if ui.button("Start").clicked() {
                                                    self.set_active_task(Some(task.id));
                                                }
                                            },
                                        );
                                    });

                                    if !task.description.is_empty() {
                                        ui.label(
                                            RichText::new(task.description.clone())
                                                .small()
                                                .color(Color32::from_rgb(98, 107, 118)),
                                        );
                                    }
                                });
                            },
                        );
                        ui.add_space(8.0);
                    }
                });
        });
    }
}

fn widget_shell_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(246, 242, 236))
        .stroke(Stroke::new(1.0, Color32::from_rgb(223, 216, 206)))
        .corner_radius(CornerRadius::same(28))
        .inner_margin(Margin::same(16))
}

pub(super) fn section_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(252, 250, 247))
        .stroke(Stroke::new(1.0, Color32::from_rgb(226, 220, 212)))
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin::same(14))
}

fn task_tile_frame(accent: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(accent.gamma_multiply(0.18))
        .stroke(Stroke::new(1.0, accent.gamma_multiply(0.55)))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::same(12))
}

fn mini_card(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Color32::from_rgb(243, 239, 233))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.label(
                RichText::new(title)
                    .small()
                    .strong()
                    .color(Color32::from_rgb(96, 105, 116)),
            );
            ui.add_space(4.0);
            add_contents(ui);
        });
}

pub(super) fn section_heading(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.label(
        RichText::new(title)
            .size(18.0)
            .strong()
            .color(Color32::from_rgb(49, 57, 66)),
    );
    ui.label(
        RichText::new(subtitle)
            .small()
            .color(Color32::from_rgb(107, 116, 126)),
    );
    ui.add_space(10.0);
}

pub(super) fn chip(ui: &mut egui::Ui, label: &str, color: Color32) {
    let text = RichText::new(label).small().color(Color32::WHITE);
    egui::Frame::new()
        .fill(color)
        .corner_radius(CornerRadius::same(255))
        .inner_margin(Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.label(text);
        });
}

fn task_label(task: &Task) -> String {
    if let Some(external_id) = &task.external_id {
        format!("#{external_id} {}", task.title)
    } else {
        task.title.clone()
    }
}

fn task_tint(task: &Task) -> Color32 {
    match task.state {
        TaskState::Active => Color32::from_rgb(92, 149, 123),
        TaskState::Planned => Color32::from_rgb(78, 129, 114),
        TaskState::Done => Color32::from_rgb(111, 136, 122),
        TaskState::Backlog => Color32::from_rgb(123, 136, 150),
    }
}
