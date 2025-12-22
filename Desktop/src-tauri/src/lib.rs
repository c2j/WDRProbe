// Learn more about Tauri commands at https://tauri.app/v1/guides/features/commands

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub type Result<T> = std::result::Result<T, anyhow::Error>;
