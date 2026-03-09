use anyhow::{Result, anyhow};
use chrono::NaiveDate;
use rusqlite::{OptionalExtension, params};

use crate::models::{Task, TaskCompletionDraft, TaskSource, TaskState};

use super::{
    Database,
    shared::{day_string, map_task, now_utc_string, parse_utc},
};

impl Database {
    pub fn seed_demo_data_if_empty(&self, today: NaiveDate) -> Result<()> {
        let mut connection = self.connection()?;
        let task_count: i64 =
            connection.query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))?;
        let meeting_count: i64 =
            connection.query_row("SELECT COUNT(*) FROM calendar_events", [], |row| row.get(0))?;
        if task_count > 0 || meeting_count > 0 {
            return Ok(());
        }

        let now = now_utc_string();
        let transaction = connection.transaction()?;
        transaction.execute(
            "INSERT INTO tasks (external_id, title, description, source, state, estimated_minutes, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![
                "12345",
                "Review scheduler UX",
                "Make sure draft tasks can be dropped into the day itinerary quickly.",
                TaskSource::AzureDevOps.as_str(),
                TaskState::Backlog.as_str(),
                90,
                now,
            ],
        )?;
        transaction.execute(
            "INSERT INTO tasks (external_id, title, description, source, state, estimated_minutes, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![
                "12346",
                "Implement activity tracker storage",
                "Track active windows locally and keep CPU overhead low.",
                TaskSource::AzureDevOps.as_str(),
                TaskState::Planned.as_str(),
                120,
                now,
            ],
        )?;
        transaction.execute(
            "INSERT INTO tasks (external_id, title, description, source, state, estimated_minutes, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![
                Option::<String>::None,
                "Write tomorrow's plan",
                "Block time for focused delivery and admin cleanup.",
                TaskSource::Local.as_str(),
                TaskState::Backlog.as_str(),
                45,
                now,
            ],
        )?;

        let day = day_string(today);
        transaction.execute(
            "INSERT INTO calendar_events (external_id, title, location, organizer, body, day, start_minute, end_minute, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'local', ?9, ?9)",
            params![
                Option::<String>::None,
                "Daily team sync",
                "Teams",
                "You",
                "Seed meeting until Outlook sync is wired in.",
                day,
                9 * 60,
                9 * 60 + 30,
                now,
            ],
        )?;
        transaction.execute(
            "INSERT INTO calendar_events (external_id, title, location, organizer, body, day, start_minute, end_minute, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'local', ?9, ?9)",
            params![
                Option::<String>::None,
                "1:1 catch-up",
                "Meeting room 2",
                "Manager",
                "Placeholder event for itinerary layout.",
                day,
                14 * 60,
                14 * 60 + 30,
                now,
            ],
        )?;
        transaction.commit()?;

        let tasks = self.list_tasks()?;
        if let Some(task) = tasks
            .iter()
            .find(|task| task.title == "Implement activity tracker storage")
        {
            self.plan_task_block(task.id, today, 10 * 60, 120)?;
        }
        Ok(())
    }

    pub fn list_tasks(&self) -> Result<Vec<Task>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, external_id, title, description, source, state, estimated_minutes,
                   logged_minutes, pending_remote_writeback_minutes, created_at, updated_at, completed_at
            FROM tasks
            ORDER BY
                CASE state
                    WHEN 'active' THEN 0
                    WHEN 'planned' THEN 1
                    WHEN 'backlog' THEN 2
                    ELSE 3
                END,
                updated_at DESC,
                title COLLATE NOCASE ASC
            ",
        )?;
        let rows = statement.query_map([], map_task)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(anyhow::Error::from)
    }

    pub fn create_local_task(
        &self,
        title: &str,
        description: &str,
        estimate_minutes: Option<i32>,
    ) -> Result<i64> {
        let connection = self.connection()?;
        let now = now_utc_string();
        connection.execute(
            "
            INSERT INTO tasks (title, description, source, state, estimated_minutes, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
            ",
            params![
                title.trim(),
                description.trim(),
                TaskSource::Local.as_str(),
                TaskState::Backlog.as_str(),
                estimate_minutes,
                now,
            ],
        )?;
        Ok(connection.last_insert_rowid())
    }

    pub fn update_task_state(&self, task_id: i64, state: TaskState) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "
            UPDATE tasks
            SET state = ?2, updated_at = ?3
            WHERE id = ?1
            ",
            params![task_id, state.as_str(), now_utc_string()],
        )?;
        Ok(())
    }

    pub fn get_task(&self, task_id: i64) -> Result<Option<Task>> {
        let connection = self.connection()?;
        connection
            .query_row(
                "
                SELECT id, external_id, title, description, source, state, estimated_minutes,
                       logged_minutes, pending_remote_writeback_minutes, created_at, updated_at, completed_at
                FROM tasks
                WHERE id = ?1
                ",
                [task_id],
                map_task,
            )
            .optional()
            .map_err(anyhow::Error::from)
    }

    pub fn set_active_task_id(&self, task_id: Option<i64>) -> Result<()> {
        let connection = self.connection()?;
        let value = task_id.map(|id| id.to_string()).unwrap_or_else(String::new);
        connection.execute(
            "
            INSERT INTO app_settings (key, value, updated_at)
            VALUES ('active_task_id', ?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
            ",
            params![value, now_utc_string()],
        )?;
        Ok(())
    }

    pub fn get_active_task_id(&self) -> Result<Option<i64>> {
        let connection = self.connection()?;
        let value: Option<String> = connection
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_task_id'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        Ok(value.and_then(|raw| raw.parse::<i64>().ok()))
    }

    pub fn complete_task(&self, task_id: i64) -> Result<TaskCompletionDraft> {
        let task = self
            .get_task(task_id)?
            .ok_or_else(|| anyhow!("task {task_id} not found"))?;

        let activity_minutes = self.sum_activity_minutes_for_task(task_id)?;
        let planned_minutes = self.sum_planned_minutes_for_task(task_id)?;
        let minutes = if activity_minutes > 0 {
            activity_minutes
        } else {
            planned_minutes
        };

        let mut connection = self.connection()?;
        let now = now_utc_string();
        let transaction = connection.transaction()?;
        transaction.execute(
            "
            UPDATE tasks
            SET state = ?2,
                completed_at = ?3,
                updated_at = ?3,
                logged_minutes = logged_minutes + ?4
            WHERE id = ?1
            ",
            params![task_id, TaskState::Done.as_str(), now, minutes],
        )?;
        transaction.execute(
            "
            INSERT INTO task_effort_logs (task_id, minutes, source, synced_remote, created_at)
            VALUES (?1, ?2, ?3, 0, ?4)
            ",
            params![task_id, minutes, "completion-rollup", now],
        )?;
        transaction.commit()?;

        if self.get_active_task_id()? == Some(task_id) {
            self.set_active_task_id(None)?;
        }

        Ok(TaskCompletionDraft {
            task_id,
            title: task.title,
            minutes,
            external_id: task.external_id,
        })
    }

    pub fn queue_remote_hours_writeback(&self, task_id: i64, minutes: i32) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "
            UPDATE tasks
            SET pending_remote_writeback_minutes = COALESCE(pending_remote_writeback_minutes, 0) + ?2,
                updated_at = ?3
            WHERE id = ?1
            ",
            params![task_id, minutes, now_utc_string()],
        )?;
        Ok(())
    }

    fn sum_activity_minutes_for_task(&self, task_id: i64) -> Result<i32> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT started_at, ended_at
            FROM activity_segments
            WHERE task_id = ?1
            ",
        )?;
        let rows = statement.query_map([task_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut total = 0i64;
        for row in rows {
            let (started_at, ended_at) = row?;
            let started_at = parse_utc(&started_at)?;
            let ended_at = parse_utc(&ended_at)?;
            total += (ended_at - started_at).num_minutes().max(0);
        }
        Ok(total.min(i32::MAX as i64) as i32)
    }

    fn sum_planned_minutes_for_task(&self, task_id: i64) -> Result<i32> {
        let connection = self.connection()?;
        let total: i64 = connection.query_row(
            "
            SELECT COALESCE(SUM(end_minute - start_minute), 0)
            FROM schedule_blocks
            WHERE task_id = ?1
            ",
            [task_id],
            |row| row.get(0),
        )?;
        Ok(total.min(i32::MAX as i64) as i32)
    }
}
