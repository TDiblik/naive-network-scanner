#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[macro_use]
extern crate lazy_static;

mod app;
mod project_selector;
mod utils;

use anyhow::anyhow;
use app::workspace::Workspace;
use eframe::egui;
use log::error;
use utils::constants::{WORKSPACE_WINDOW_HEIGHT, WORKSPACE_WINDOW_WIDTH};

fn main() -> anyhow::Result<()> {
    utils::env::init()?;
    utils::logging::init()?;
    // TODO: Perform runtime checks

    // TODO: If debug => open workspace with custom env for testing
    //       else => open project selector and open desired project

    // TODO: Implement and Open project selector
    // eframe::run_native(
    //     "Naive Network Scanner - Project Selector",
    //     eframe::NativeOptions {
    //         initial_window_size: Some(egui::vec2(550.0, 440.0)),
    //         resizable: false,
    //         ..Default::default()
    //     },
    //     Box::new(|_cc| Box::<ProjectSelector>::default()),
    // )
    // .expect("Unable to render project selector.");

    let new_id = uuid::Uuid::new_v4();
    let result = eframe::run_native(
        &format!("Naive Network Scanner - Workspace {}", new_id),
        eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(WORKSPACE_WINDOW_WIDTH, WORKSPACE_WINDOW_HEIGHT)),
            ..Default::default()
        },
        Box::new(move |_cc| Box::new(Workspace::new(new_id))),
    );
    if result.is_err() {
        error!("{:?}", result);
        return Err(anyhow!("Workspace ended with error. More info in logs."));
    }

    Ok(())
}
