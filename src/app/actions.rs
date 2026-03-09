use crate::models::{CalendarEvent, ScheduleBlock, TaskCompletionDraft, TaskState};

use super::{DragPayload, ProductivApp, SelectedItem};

impl ProductivApp {
    pub(super) fn refresh_all(&mut self) {
        match self.database.list_tasks() {
            Ok(tasks) => self.tasks = tasks,
            Err(error) => self.status_message = Some(format!("Failed to load tasks: {error}")),
        }
        match self.database.list_schedule_for_day(self.selected_day) {
            Ok(blocks) => self.schedule_blocks = blocks,
            Err(error) => {
                self.status_message = Some(format!("Failed to load schedule blocks: {error}"))
            }
        }
        match self
            .database
            .list_calendar_events_for_day(self.selected_day)
        {
            Ok(events) => self.calendar_events = events,
            Err(error) => self.status_message = Some(format!("Failed to load meetings: {error}")),
        }
        match self.database.list_recent_activity(12) {
            Ok(activity) => self.recent_activity = activity,
            Err(error) => self.status_message = Some(format!("Failed to load activity: {error}")),
        }
        self.last_refresh = std::time::Instant::now();
    }

    pub(super) fn set_active_task(&mut self, task_id: Option<i64>) {
        if let Err(error) = self.database.set_active_task_id(task_id) {
            self.status_message = Some(format!("Failed to persist active task: {error}"));
            return;
        }
        self.runtime.set_active_task_id(task_id);
        if let Some(task_id) = task_id {
            let _ = self.database.update_task_state(task_id, TaskState::Active);
        }
        self.refresh_all();
    }

    pub(super) fn create_task(&mut self) {
        let title = self.draft_task_title.trim();
        if title.is_empty() {
            self.status_message = Some("Task title is required.".to_owned());
            return;
        }
        let estimate_minutes = Some((self.draft_task_estimate_hours * 60.0).round() as i32);
        match self.database.create_local_task(
            title,
            self.draft_task_description.trim(),
            estimate_minutes,
        ) {
            Ok(_) => {
                self.draft_task_title.clear();
                self.draft_task_description.clear();
                self.draft_task_estimate_hours = 1.0;
                self.status_message = Some("Task drafted into the local list.".to_owned());
                self.refresh_all();
            }
            Err(error) => {
                self.status_message = Some(format!("Failed to draft task: {error}"));
            }
        }
    }

    pub(super) fn complete_task(&mut self, task_id: i64) {
        match self.database.complete_task(task_id) {
            Ok(summary) => {
                self.completion_prompt = Some(summary);
                self.refresh_all();
            }
            Err(error) => {
                self.status_message = Some(format!("Failed to close task: {error}"));
            }
        }
    }

    pub(super) fn handle_drop(&mut self, payload: DragPayload, start_minute: i32) {
        let result = match payload {
            DragPayload::Task(task_id) => self.database.plan_task_block(
                task_id,
                self.selected_day,
                start_minute,
                self.default_plan_minutes,
            ),
            DragPayload::Block(block_id) => self
                .database
                .move_schedule_block(block_id, self.selected_day, start_minute)
                .map(|_| block_id),
        };

        match result {
            Ok(_) => {
                self.refresh_all();
                self.status_message = Some(format!(
                    "Planned block at {} for {} minutes.",
                    super::planner::minute_label(start_minute),
                    self.default_plan_minutes
                ));
            }
            Err(error) => {
                self.status_message = Some(format!("Failed to update itinerary: {error}"));
            }
        }
    }

    pub(super) fn save_config(&mut self) {
        if self.config_draft.activity_poll_seconds == 0 {
            self.config_draft.activity_poll_seconds = 1;
        }
        if self.config_draft.idle_threshold_minutes == 0 {
            self.config_draft.idle_threshold_minutes = 1;
        }

        match self.database.save_app_config(&self.config_draft) {
            Ok(_) => {
                self.status_message = Some(
                    "Configuration saved to LocalAppData. PAT sync is wired for storage, not remote calls yet."
                        .to_owned(),
                );
            }
            Err(error) => {
                self.status_message = Some(format!("Failed to save config: {error}"));
            }
        }
    }

    pub(super) fn selected_block(&self) -> Option<ScheduleBlock> {
        match self.selected_item {
            Some(SelectedItem::Block(id)) => self
                .schedule_blocks
                .iter()
                .find(|block| block.id == id)
                .cloned(),
            _ => None,
        }
    }

    pub(super) fn selected_meeting(&self) -> Option<CalendarEvent> {
        match self.selected_item {
            Some(SelectedItem::Meeting(id)) => self
                .calendar_events
                .iter()
                .find(|meeting| meeting.id == id)
                .cloned(),
            _ => None,
        }
    }

    pub(super) fn completion_prompt(&self) -> Option<TaskCompletionDraft> {
        self.completion_prompt.clone()
    }
}
