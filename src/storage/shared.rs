use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};

use crate::models::{
    ActivitySegment, CalendarEvent, ScheduleBlock, ScheduleKind, Task, TaskSource, TaskState,
};

pub(super) fn map_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        external_id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        source: TaskSource::from_db(&row.get::<_, String>(4)?),
        state: TaskState::from_db(&row.get::<_, String>(5)?),
        estimated_minutes: row.get(6)?,
        logged_minutes: row.get(7)?,
        pending_remote_writeback_minutes: row.get(8)?,
        created_at: parse_utc(&row.get::<_, String>(9)?).map_err(to_sql_error)?,
        updated_at: parse_utc(&row.get::<_, String>(10)?).map_err(to_sql_error)?,
        completed_at: row
            .get::<_, Option<String>>(11)?
            .map(|value| parse_utc(&value).map_err(to_sql_error))
            .transpose()?,
    })
}

pub(super) fn map_schedule_block(row: &rusqlite::Row<'_>) -> rusqlite::Result<ScheduleBlock> {
    Ok(ScheduleBlock {
        id: row.get(0)?,
        task_id: row.get(1)?,
        title: row.get(2)?,
        kind: ScheduleKind::from_db(&row.get::<_, String>(3)?),
        day: parse_day(&row.get::<_, String>(4)?).map_err(to_sql_error)?,
        start_minute: row.get(5)?,
        end_minute: row.get(6)?,
        notes: row.get(7)?,
        created_at: parse_utc(&row.get::<_, String>(8)?).map_err(to_sql_error)?,
        updated_at: parse_utc(&row.get::<_, String>(9)?).map_err(to_sql_error)?,
    })
}

pub(super) fn map_calendar_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<CalendarEvent> {
    Ok(CalendarEvent {
        id: row.get(0)?,
        external_id: row.get(1)?,
        title: row.get(2)?,
        location: row.get(3)?,
        organizer: row.get(4)?,
        body: row.get(5)?,
        day: parse_day(&row.get::<_, String>(6)?).map_err(to_sql_error)?,
        start_minute: row.get(7)?,
        end_minute: row.get(8)?,
        source: row.get(9)?,
    })
}

pub(super) fn map_activity_segment(row: &rusqlite::Row<'_>) -> rusqlite::Result<ActivitySegment> {
    Ok(ActivitySegment {
        id: row.get(0)?,
        task_id: row.get(1)?,
        started_at: parse_utc(&row.get::<_, String>(2)?).map_err(to_sql_error)?,
        ended_at: parse_utc(&row.get::<_, String>(3)?).map_err(to_sql_error)?,
        process_name: row.get(4)?,
        exe_path: row.get(5)?,
        window_title: row.get(6)?,
        window_class: row.get(7)?,
        idle_seconds: row.get(8)?,
        link_reason: row.get(9)?,
    })
}

pub(super) fn parse_utc(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("invalid RFC3339 timestamp: {value}"))?
        .with_timezone(&Utc))
}

pub(super) fn now_utc_string() -> String {
    to_utc_string(Utc::now())
}

pub(super) fn to_utc_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

pub(super) fn day_string(value: NaiveDate) -> String {
    value.format("%Y-%m-%d").to_string()
}

fn parse_day(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .with_context(|| format!("invalid day value: {value}"))
}

fn to_sql_error(error: anyhow::Error) -> rusqlite::Error {
    let io_error = std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string());
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(io_error))
}
