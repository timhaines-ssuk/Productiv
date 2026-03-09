use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::{DateTime, Duration as ChronoDuration, Utc};

use crate::storage::Database;

use super::windows_tracker::capture_window_snapshot;

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const IDLE_THRESHOLD_SECONDS: u64 = 5 * 60;

#[derive(Clone, Debug)]
pub struct LiveTrackerStatus {
    pub available: bool,
    pub tracking_note: String,
    pub process_name: String,
    pub window_title: String,
    pub window_class: String,
    pub idle_seconds: u64,
    pub current_task_id: Option<i64>,
    pub last_sample_at: DateTime<Utc>,
}

impl Default for LiveTrackerStatus {
    fn default() -> Self {
        Self {
            available: false,
            tracking_note: "Tracker starting".to_owned(),
            process_name: String::new(),
            window_title: String::new(),
            window_class: String::new(),
            idle_seconds: 0,
            current_task_id: None,
            last_sample_at: Utc::now(),
        }
    }
}

pub struct BackgroundRuntime {
    active_task_id: Arc<Mutex<Option<i64>>>,
    status: Arc<Mutex<LiveTrackerStatus>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl BackgroundRuntime {
    pub fn start(database: Database, active_task_id: Arc<Mutex<Option<i64>>>) -> Self {
        let status = Arc::new(Mutex::new(LiveTrackerStatus::default()));
        let stop = Arc::new(AtomicBool::new(false));

        let thread_status = Arc::clone(&status);
        let thread_stop = Arc::clone(&stop);
        let thread_active_task_id = Arc::clone(&active_task_id);

        let handle = thread::spawn(move || {
            tracking_loop(database, thread_active_task_id, thread_status, thread_stop);
        });

        Self {
            active_task_id,
            status,
            stop,
            handle: Some(handle),
        }
    }

    pub fn set_active_task_id(&self, task_id: Option<i64>) {
        if let Ok(mut active_task) = self.active_task_id.lock() {
            *active_task = task_id;
        }
    }

    pub fn active_task_id(&self) -> Option<i64> {
        self.active_task_id.lock().ok().and_then(|task| *task)
    }

    pub fn status(&self) -> LiveTrackerStatus {
        self.status
            .lock()
            .map(|status| status.clone())
            .unwrap_or_default()
    }
}

impl Drop for BackgroundRuntime {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct WindowSnapshot {
    pub captured_at: DateTime<Utc>,
    pub process_name: String,
    pub exe_path: Option<String>,
    pub window_title: String,
    pub window_class: String,
    pub idle_seconds: u64,
}

#[derive(Clone, Debug)]
struct OpenSegment {
    started_at: DateTime<Utc>,
    task_id: Option<i64>,
    link_reason: String,
    snapshot: WindowSnapshot,
}

fn tracking_loop(
    database: Database,
    active_task_id: Arc<Mutex<Option<i64>>>,
    status: Arc<Mutex<LiveTrackerStatus>>,
    stop: Arc<AtomicBool>,
) {
    let mut current: Option<OpenSegment> = None;

    while !stop.load(Ordering::Relaxed) {
        let now = Utc::now();
        let active_task = active_task_id.lock().ok().and_then(|task| *task);

        match capture_window_snapshot() {
            Ok(Some(snapshot)) if snapshot.idle_seconds >= IDLE_THRESHOLD_SECONDS => {
                if let Some(segment) = current.take() {
                    let idle_started_at =
                        now - ChronoDuration::seconds(snapshot.idle_seconds as i64);
                    let _ = flush_segment(&database, segment, idle_started_at);
                }
                update_status(&status, true, "Idle", &snapshot, active_task);
            }
            Ok(Some(snapshot)) => {
                let needs_rollover = current
                    .as_ref()
                    .map(|segment| {
                        segment.task_id != active_task
                            || segment.snapshot.process_name != snapshot.process_name
                            || segment.snapshot.window_title != snapshot.window_title
                            || segment.snapshot.window_class != snapshot.window_class
                    })
                    .unwrap_or(true);

                if needs_rollover {
                    if let Some(segment) = current.take() {
                        let _ = flush_segment(&database, segment, snapshot.captured_at);
                    }
                    current = Some(OpenSegment {
                        started_at: snapshot.captured_at,
                        task_id: active_task,
                        link_reason: if active_task.is_some() {
                            "manual-active-task".to_owned()
                        } else {
                            "unlinked".to_owned()
                        },
                        snapshot: snapshot.clone(),
                    });
                }

                update_status(
                    &status,
                    true,
                    "Tracking foreground window",
                    &snapshot,
                    active_task,
                );
            }
            Ok(None) => {
                update_status(
                    &status,
                    false,
                    "No foreground window",
                    &WindowSnapshot {
                        captured_at: now,
                        process_name: String::new(),
                        exe_path: None,
                        window_title: String::new(),
                        window_class: String::new(),
                        idle_seconds: 0,
                    },
                    active_task,
                );
            }
            Err(error) => {
                update_status(
                    &status,
                    false,
                    &format!("Tracker error: {error}"),
                    &WindowSnapshot {
                        captured_at: now,
                        process_name: String::new(),
                        exe_path: None,
                        window_title: String::new(),
                        window_class: String::new(),
                        idle_seconds: 0,
                    },
                    active_task,
                );
            }
        }

        thread::sleep(POLL_INTERVAL);
    }

    if let Some(segment) = current.take() {
        let _ = flush_segment(&database, segment, Utc::now());
    }
}

fn flush_segment(
    database: &Database,
    segment: OpenSegment,
    ended_at: DateTime<Utc>,
) -> anyhow::Result<()> {
    database.insert_activity_segment(
        segment.task_id,
        segment.started_at,
        ended_at,
        &segment.snapshot.process_name,
        segment.snapshot.exe_path.as_deref(),
        &segment.snapshot.window_title,
        &segment.snapshot.window_class,
        segment.snapshot.idle_seconds as i64,
        &segment.link_reason,
    )
}

fn update_status(
    status: &Arc<Mutex<LiveTrackerStatus>>,
    available: bool,
    tracking_note: &str,
    snapshot: &WindowSnapshot,
    current_task_id: Option<i64>,
) {
    if let Ok(mut state) = status.lock() {
        *state = LiveTrackerStatus {
            available,
            tracking_note: tracking_note.to_owned(),
            process_name: snapshot.process_name.clone(),
            window_title: snapshot.window_title.clone(),
            window_class: snapshot.window_class.clone(),
            idle_seconds: snapshot.idle_seconds,
            current_task_id,
            last_sample_at: snapshot.captured_at,
        };
    }
}
