#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod project_selector;
mod utils;

use anyhow::anyhow;
use app::workspace::Workspace;
use eframe::egui;
use log::error;

fn main() -> anyhow::Result<()> {
    utils::env::init()?;
    utils::logging::init()?;
    // TODO: Perform runtime checks

    // TODO: If debug => open workspace with custom env for testing
    //       else => open project selector and open desired project

    // TODO: Implement and Open project selector
    // eframe::run_native(
    //     "TEEF - Project Selector",
    //     eframe::NativeOptions {
    //         initial_window_size: Some(egui::vec2(550.0, 440.0)),
    //         resizable: false,
    //         ..Default::default()
    //     },
    //     Box::new(|_cc| Box::<ProjectSelector>::default()),
    // )
    // .expect("Unable to render project selector.");

    let result = eframe::run_native(
        "TEEF - Workspace {ID}",
        eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(1250.0, 1080.0)),
            ..Default::default()
        },
        Box::new(|_cc| Box::<Workspace>::default()),
    );
    if result.is_err() {
        error!("{:?}", result);
        return Err(anyhow!("Workspace ended with error. More info in logs."));
    }

    Ok(())
}
