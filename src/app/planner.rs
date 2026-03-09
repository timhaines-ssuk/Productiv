use eframe::egui::{self, Align2, Color32, FontId, Id, RichText, Sense, Stroke, StrokeKind};

use super::{
    DAY_END_MINUTE, DAY_START_MINUTE, DragPayload, ProductivApp, SLOT_HEIGHT, SLOT_MINUTES,
    SelectedItem,
};

impl ProductivApp {
    pub(super) fn show_planner(&mut self, ctx: &egui::Context) {
        let schedule_blocks = self.schedule_blocks.clone();
        let calendar_events = self.calendar_events.clone();

        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(egui::Margin::same(12)))
            .show(ctx, |ui| {
                ui.heading("Full Day Itinerary");
                ui.label("Drag task cards from the left into a time slot. Drag an existing planned block to reposition it.");
                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for minute in (DAY_START_MINUTE..DAY_END_MINUTE).step_by(SLOT_MINUTES as usize)
                    {
                        let starting_blocks: Vec<_> = schedule_blocks
                            .iter()
                            .filter(|block| block.start_minute == minute)
                            .cloned()
                            .collect();
                        let starting_meetings: Vec<_> = calendar_events
                            .iter()
                            .filter(|meeting| meeting.start_minute == minute)
                            .cloned()
                            .collect();

                        ui.horizontal(|ui| {
                            ui.set_min_height(SLOT_HEIGHT);
                            ui.set_height(SLOT_HEIGHT);
                            ui.allocate_ui(egui::vec2(84.0, SLOT_HEIGHT), |ui| {
                                ui.label(RichText::new(minute_label(minute)).strong());
                            });

                            let lane_width = ui.available_width();
                            let (slot_rect, slot_response) = ui
                                .allocate_exact_size(egui::vec2(lane_width, SLOT_HEIGHT), Sense::click());
                            let painter = ui.painter_at(slot_rect);

                            let slot_fill =
                                if slot_response.dnd_hover_payload::<DragPayload>().is_some() {
                                    Color32::from_rgb(46, 70, 102)
                                } else if minute % (2 * SLOT_MINUTES) == 0 {
                                    Color32::from_rgb(22, 28, 38)
                                } else {
                                    Color32::from_rgb(17, 22, 31)
                                };
                            painter.rect_filled(slot_rect, 8.0, slot_fill);
                            painter.rect_stroke(
                                slot_rect,
                                8.0,
                                Stroke::new(1.0, Color32::from_rgb(53, 64, 84)),
                                StrokeKind::Inside,
                            );

                            if let Some(payload) =
                                slot_response.dnd_release_payload::<DragPayload>()
                            {
                                self.handle_drop(*payload, minute);
                            }

                            for (index, meeting) in starting_meetings.iter().enumerate() {
                                let duration_slots =
                                    ((meeting.end_minute - meeting.start_minute) / SLOT_MINUTES)
                                        .max(1);
                                let rect = block_rect(slot_rect, index as f32, duration_slots);
                                let response =
                                    ui.interact(rect, Id::new(("meeting", meeting.id)), Sense::click());
                                if response.clicked() {
                                    self.selected_item = Some(SelectedItem::Meeting(meeting.id));
                                }
                                painter.rect_filled(rect, 8.0, Color32::from_rgb(160, 101, 44));
                                painter.text(
                                    rect.left_top() + egui::vec2(10.0, 10.0),
                                    Align2::LEFT_TOP,
                                    format!(
                                        "{}\n{} - {}",
                                        meeting.title,
                                        minute_label(meeting.start_minute),
                                        minute_label(meeting.end_minute)
                                    ),
                                    FontId::proportional(14.0),
                                    Color32::WHITE,
                                );
                            }

                            for (index, block) in starting_blocks.iter().enumerate() {
                                let duration_slots =
                                    ((block.end_minute - block.start_minute) / SLOT_MINUTES)
                                        .max(1);
                                let rect = block_rect(
                                    slot_rect,
                                    (starting_meetings.len() + index) as f32,
                                    duration_slots,
                                );
                                let response = ui.interact(
                                    rect,
                                    Id::new(("block", block.id)),
                                    Sense::click_and_drag(),
                                );
                                if response.clicked() {
                                    self.selected_item = Some(SelectedItem::Block(block.id));
                                }
                                if response.drag_started() {
                                    response.dnd_set_drag_payload(DragPayload::Block(block.id));
                                }
                                painter.rect_filled(rect, 8.0, Color32::from_rgb(57, 120, 101));
                                painter.text(
                                    rect.left_top() + egui::vec2(10.0, 10.0),
                                    Align2::LEFT_TOP,
                                    format!(
                                        "{}\n{} - {}",
                                        block.title,
                                        minute_label(block.start_minute),
                                        minute_label(block.end_minute)
                                    ),
                                    FontId::proportional(14.0),
                                    Color32::WHITE,
                                );
                            }
                        });
                        ui.add_space(4.0);
                    }
                });
            });
    }
}

fn block_rect(slot_rect: egui::Rect, stack_index: f32, duration_slots: i32) -> egui::Rect {
    let horizontal_offset = 8.0 + stack_index * 12.0;
    let width = (slot_rect.width() - horizontal_offset - 10.0).max(120.0);
    let top = slot_rect.top() + 5.0 + stack_index * 4.0;
    let height = ((duration_slots as f32) * SLOT_HEIGHT - 10.0).max(SLOT_HEIGHT - 10.0);
    egui::Rect::from_min_size(
        egui::pos2(slot_rect.left() + horizontal_offset, top),
        egui::vec2(width, height),
    )
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
