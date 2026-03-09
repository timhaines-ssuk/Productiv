use chrono::{Local, Timelike};
use eframe::egui::{self, Align, Color32, CornerRadius, Id, Margin, RichText, Stroke};

use crate::models::{CalendarEvent, ScheduleBlock};

use super::{DAY_END_MINUTE, DAY_START_MINUTE, DragPayload, ProductivApp, SLOT_MINUTES};
use crate::app::cards::{section_frame, section_heading};

impl ProductivApp {
    pub(super) fn show_day_plan_card(&mut self, ui: &mut egui::Ui) {
        let blocks = self.schedule_blocks.clone();
        let meetings = self.calendar_events.clone();

        section_frame().show(ui, |ui| {
            section_heading(
                ui,
                "Day Plan",
                "Drop a task on a time slot below. Existing task blocks can be dragged to reschedule.",
            );
            ui.horizontal(|ui| {
                ui.label("Default block");
                ui.add(
                    egui::Slider::new(&mut self.default_plan_minutes, 30..=180)
                        .step_by(30.0)
                        .suffix(" min"),
                );
            });
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(360.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for minute in (DAY_START_MINUTE..DAY_END_MINUTE).step_by(SLOT_MINUTES as usize)
                    {
                        let blocks_at: Vec<ScheduleBlock> = blocks
                            .iter()
                            .filter(|block| block.start_minute == minute)
                            .cloned()
                            .collect();
                        let meetings_at: Vec<CalendarEvent> = meetings
                            .iter()
                            .filter(|meeting| meeting.start_minute == minute)
                            .cloned()
                            .collect();
                        let item_count = blocks_at.len() + meetings_at.len();
                        let row_height = if item_count == 0 {
                            42.0
                        } else {
                            18.0 + (item_count as f32 * 66.0)
                        };

                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [54.0, row_height],
                                egui::Label::new(
                                    RichText::new(minute_label(minute))
                                        .small()
                                        .strong()
                                        .color(Color32::from_rgb(107, 116, 126)),
                                ),
                            );

                            let drop_frame = egui::Frame::new()
                                .fill(Color32::from_rgb(239, 235, 228))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(221, 214, 204)))
                                .corner_radius(CornerRadius::same(14))
                                .inner_margin(Margin::same(10));
                            let (_, payload) =
                                ui.dnd_drop_zone::<DragPayload, _>(drop_frame, |ui| {
                                    ui.set_min_height(row_height - 2.0);
                                    if item_count == 0 {
                                        ui.label(
                                            RichText::new("Drop task here")
                                                .small()
                                                .color(Color32::from_rgb(137, 128, 118)),
                                        );
                                    }

                                    for meeting in &meetings_at {
                                        itinerary_tile_frame(Color32::from_rgb(230, 202, 173))
                                            .show(ui, |ui| {
                                                ui.label(RichText::new(&meeting.title).strong());
                                                ui.label(
                                                    RichText::new(format!(
                                                        "{} - {}",
                                                        minute_label(meeting.start_minute),
                                                        minute_label(meeting.end_minute)
                                                    ))
                                                    .small(),
                                                );
                                                if !meeting.location.is_empty() {
                                                    ui.label(
                                                        RichText::new(&meeting.location)
                                                            .small()
                                                            .color(Color32::from_rgb(120, 99, 72)),
                                                    );
                                                }
                                            });
                                        ui.add_space(6.0);
                                    }

                                    for block in &blocks_at {
                                        ui.dnd_drag_source(
                                            Id::new(("block", block.id)),
                                            DragPayload::Block(block.id),
                                            |ui| {
                                                itinerary_tile_frame(Color32::from_rgb(
                                                    198, 219, 209,
                                                ))
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.vertical(|ui| {
                                                            ui.label(
                                                                RichText::new(&block.title)
                                                                    .strong(),
                                                            );
                                                            ui.label(
                                                                RichText::new(format!(
                                                                    "{} - {} · {}",
                                                                    minute_label(
                                                                        block.start_minute
                                                                    ),
                                                                    minute_label(block.end_minute),
                                                                    minutes_label(
                                                                        (block.end_minute
                                                                            - block.start_minute)
                                                                            as i64
                                                                    )
                                                                ))
                                                                .small(),
                                                            );
                                                        });
                                                        ui.with_layout(
                                                            egui::Layout::right_to_left(
                                                                Align::Center,
                                                            ),
                                                            |ui| {
                                                                if ui.button("Clear").clicked() {
                                                                    if let Err(error) = self
                                                                        .database
                                                                        .delete_schedule_block(
                                                                            block.id,
                                                                        )
                                                                    {
                                                                        self.status_message = Some(
                                                                            format!(
                                                                                "Failed to clear block: {error}"
                                                                            ),
                                                                        );
                                                                    } else {
                                                                        self.refresh_all();
                                                                    }
                                                                }
                                                                if let Some(task_id) = block.task_id
                                                                {
                                                                    if ui.button("Start").clicked()
                                                                    {
                                                                        self.set_active_task(Some(
                                                                            task_id,
                                                                        ));
                                                                    }
                                                                }
                                                            },
                                                        );
                                                    });
                                                });
                                            },
                                        );
                                        ui.add_space(6.0);
                                    }
                                });

                            if let Some(payload) = payload {
                                self.handle_drop(*payload, minute);
                            }
                        });
                        ui.add_space(6.0);
                    }
                });
        });
    }

    pub(super) fn show_activity_card(&mut self, ui: &mut egui::Ui) {
        let tracker = self.runtime.status();

        section_frame().show(ui, |ui| {
            section_heading(
                ui,
                "Recent Activity",
                "Low-overhead active window tracking stored locally.",
            );
            ui.label(
                RichText::new(format!(
                    "{} · idle {}s",
                    tracker.process_name, tracker.idle_seconds
                ))
                .small()
                .color(Color32::from_rgb(103, 111, 120)),
            );
            if !tracker.window_title.is_empty() {
                ui.label(
                    RichText::new(&tracker.window_title)
                        .small()
                        .color(Color32::from_rgb(103, 111, 120)),
                );
            }
            if !tracker.window_class.is_empty() {
                ui.label(
                    RichText::new(format!("Class: {}", tracker.window_class))
                        .small()
                        .color(Color32::from_rgb(103, 111, 120)),
                );
            }
            if let Some(task_id) = tracker.current_task_id {
                ui.label(
                    RichText::new(format!("Linked task id: {task_id}"))
                        .small()
                        .color(Color32::from_rgb(103, 111, 120)),
                );
            }
            ui.label(
                RichText::new(format!(
                    "Last sample {}",
                    tracker
                        .last_sample_at
                        .with_timezone(&Local)
                        .format("%H:%M:%S")
                ))
                .small()
                .color(Color32::from_rgb(103, 111, 120)),
            );
            ui.add_space(8.0);

            for segment in self.recent_activity.iter().take(4) {
                itinerary_tile_frame(Color32::from_rgb(241, 237, 231)).show(ui, |ui| {
                    ui.label(format!(
                        "{} - {}",
                        segment.started_at.with_timezone(&Local).format("%H:%M"),
                        segment.ended_at.with_timezone(&Local).format("%H:%M")
                    ));
                    ui.label(
                        RichText::new(format!(
                            "{} | {}",
                            segment.process_name, segment.window_title
                        ))
                        .small(),
                    );
                });
                ui.add_space(6.0);
            }
        });
    }

    pub(super) fn current_meeting(&self) -> Option<CalendarEvent> {
        if self.selected_day != Local::now().date_naive() {
            return None;
        }

        let now = Local::now();
        let minute_now = (now.hour() as i32 * 60) + now.minute() as i32;
        self.calendar_events
            .iter()
            .find(|meeting| meeting.start_minute <= minute_now && minute_now < meeting.end_minute)
            .cloned()
    }

    pub(super) fn next_meeting(&self) -> Option<CalendarEvent> {
        let day = if self.selected_day == Local::now().date_naive() {
            Local::now().date_naive()
        } else {
            self.selected_day
        };
        if self.selected_day != day {
            return None;
        }

        let now = Local::now();
        let minute_now = (now.hour() as i32 * 60) + now.minute() as i32;
        self.calendar_events
            .iter()
            .find(|meeting| meeting.start_minute > minute_now)
            .cloned()
    }
}

fn itinerary_tile_frame(fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, fill.gamma_multiply(0.8)))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(10))
}

pub(super) fn minute_label(minute: i32) -> String {
    let hour = minute.div_euclid(60);
    let minute = minute.rem_euclid(60);
    format!("{hour:02}:{minute:02}")
}

pub(super) fn minutes_label(minutes: i64) -> String {
    let hours = minutes / 60;
    let remainder = minutes % 60;
    if hours > 0 {
        format!("{hours}h {remainder}m")
    } else {
        format!("{remainder}m")
    }
}
