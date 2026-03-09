use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::models::ActivitySegment;

use super::{
    Database,
    shared::{map_activity_segment, now_utc_string, to_utc_string},
};

impl Database {
    pub fn list_recent_activity(&self, limit: usize) -> Result<Vec<ActivitySegment>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "
            SELECT id, task_id, started_at, ended_at, process_name, exe_path, window_title, window_class, idle_seconds, link_reason
            FROM activity_segments
            ORDER BY started_at DESC
            LIMIT ?1
            ",
        )?;
        let rows = statement.query_map([limit as i64], map_activity_segment)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(anyhow::Error::from)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_activity_segment(
        &self,
        task_id: Option<i64>,
        started_at: DateTime<Utc>,
        ended_at: DateTime<Utc>,
        process_name: &str,
        exe_path: Option<&str>,
        window_title: &str,
        window_class: &str,
        idle_seconds: i64,
        link_reason: &str,
    ) -> Result<()> {
        if ended_at <= started_at {
            return Ok(());
        }

        let connection = self.connection()?;
        connection.execute(
            "
            INSERT INTO activity_segments (
                task_id, started_at, ended_at, process_name, exe_path, window_title, window_class,
                idle_seconds, link_reason, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ",
            params![
                task_id,
                to_utc_string(started_at),
                to_utc_string(ended_at),
                process_name,
                exe_path,
                window_title,
                window_class,
                idle_seconds,
                link_reason,
                now_utc_string(),
            ],
        )?;
        Ok(())
    }
}
