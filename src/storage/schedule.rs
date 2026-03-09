use anyhow::{Result, anyhow};
use chrono::NaiveDate;
use rusqlite::{OptionalExtension, params};

use crate::models::{CalendarEvent, ScheduleBlock, ScheduleKind, TaskState};

use super::{
    Database,
    shared::{day_string, map_calendar_event, map_schedule_block, now_utc_string},
};

impl Database {
    pub fn list_schedule_for_day(&self, day: NaiveDate) -> Result<Vec<ScheduleBlock>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, task_id, title, block_kind, day, start_minute, end_minute, notes, created_at, updated_at
            FROM schedule_blocks
            WHERE day = ?1
            ORDER BY start_minute ASC, end_minute ASC, id ASC
            ",
        )?;
        let rows = statement.query_map([day_string(day)], map_schedule_block)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(anyhow::Error::from)
    }

    pub fn get_schedule_block(&self, id: i64) -> Result<Option<ScheduleBlock>> {
        let connection = self.connection()?;
        connection
            .query_row(
                "
                SELECT id, task_id, title, block_kind, day, start_minute, end_minute, notes, created_at, updated_at
                FROM schedule_blocks
                WHERE id = ?1
                ",
                [id],
                map_schedule_block,
            )
            .optional()
            .map_err(anyhow::Error::from)
    }

    pub fn list_calendar_events_for_day(&self, day: NaiveDate) -> Result<Vec<CalendarEvent>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, external_id, title, location, organizer, body, day, start_minute, end_minute, source
            FROM calendar_events
            WHERE day = ?1
            ORDER BY start_minute ASC, end_minute ASC, id ASC
            ",
        )?;
        let rows = statement.query_map([day_string(day)], map_calendar_event)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(anyhow::Error::from)
    }

    pub fn plan_task_block(
        &self,
        task_id: i64,
        day: NaiveDate,
        start_minute: i32,
        duration_minutes: i32,
    ) -> Result<i64> {
        let connection = self.connection()?;
        let task = self
            .get_task(task_id)?
            .ok_or_else(|| anyhow!("task {task_id} not found"))?;
        let now = now_utc_string();
        connection.execute(
            "
            INSERT INTO schedule_blocks (task_id, title, block_kind, day, start_minute, end_minute, notes, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, '', ?7, ?7)
            ",
            params![
                task_id,
                task.title,
                ScheduleKind::Task.as_str(),
                day_string(day),
                start_minute,
                (start_minute + duration_minutes).min(24 * 60),
                now,
            ],
        )?;
        if task.state == TaskState::Backlog {
            self.update_task_state(task_id, TaskState::Planned)?;
        }
        Ok(connection.last_insert_rowid())
    }

    pub fn move_schedule_block(
        &self,
        block_id: i64,
        day: NaiveDate,
        start_minute: i32,
    ) -> Result<()> {
        let existing = self
            .get_schedule_block(block_id)?
            .ok_or_else(|| anyhow!("schedule block {block_id} not found"))?;
        let duration = existing.end_minute - existing.start_minute;
        let connection = self.connection()?;
        connection.execute(
            "
            UPDATE schedule_blocks
            SET day = ?2, start_minute = ?3, end_minute = ?4, updated_at = ?5
            WHERE id = ?1
            ",
            params![
                block_id,
                day_string(day),
                start_minute,
                (start_minute + duration).min(24 * 60),
                now_utc_string(),
            ],
        )?;
        Ok(())
    }

    pub fn delete_schedule_block(&self, block_id: i64) -> Result<()> {
        let connection = self.connection()?;
        connection.execute("DELETE FROM schedule_blocks WHERE id = ?1", [block_id])?;
        Ok(())
    }
}
