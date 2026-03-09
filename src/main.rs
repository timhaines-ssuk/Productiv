mod app;
mod models;
mod services;
mod storage;

use std::sync::{Arc, Mutex};

use anyhow::Context;
use chrono::Local;
use eframe::egui;

use app::ProductivApp;
use services::BackgroundRuntime;
use storage::Database;

fn main() -> anyhow::Result<()> {
    let database = Database::new().context("failed to open local Productiv database")?;
    database
        .seed_demo_data_if_empty(Local::now().date_naive())
        .context("failed to seed local demo data")?;

    let active_task_id = Arc::new(Mutex::new(database.get_active_task_id()?));
    let runtime = BackgroundRuntime::start(database.clone(), Arc::clone(&active_task_id));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1.0, 1.0])
            .with_min_inner_size([1.0, 1.0])
            .with_max_inner_size([1.0, 1.0])
            .with_position([-10_000.0, -10_000.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_visible(false)
            .with_taskbar(false)
            .with_title("Productiv Host"),
        ..Default::default()
    };

    eframe::run_native(
        "Productiv",
        native_options,
        Box::new(move |cc| Ok(Box::new(ProductivApp::new(cc, database, runtime)))),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))
}
