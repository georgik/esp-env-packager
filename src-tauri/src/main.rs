// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use dirs;

mod app_state;
use app_state::{AppState, BuilderState};

mod zip_archiver;
use zip_archiver::{zip_dir, unzip};

use serde::Serialize;
use thiserror;
use tauri::{State, Window};

// Create a custom Error that we can return in Results
#[derive(Debug, thiserror::Error)]
enum Error {
    // Implement std::io::Error for our Error enum
    #[error(transparent)]
    Io(#[from] std::io::Error),
    // Add a PoisonError, but we implement it manually later
    #[error("the mutex was poisoned")]
    PoisonError(String),
}

#[tauri::command]
async fn abort_build(state_mutex: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    let mut state = state_mutex.lock().unwrap();
    state.builder = BuilderState::Abort;
    Ok("ok".to_string())
}

// Command to copress directories into a archive file.
#[tauri::command]
async fn compress(window: Window, app: tauri::AppHandle, state_mutex: State<'_, Mutex<AppState>>, source_path: String, target_path:String) -> Result<String, ()> {
    let method = zip::CompressionMethod::Deflated;

    {
        let mut state = state_mutex.lock().unwrap();
        state.builder = BuilderState::Running;
    }

    let result = zip_dir(window,app.clone(), source_path.as_str(), target_path.as_str(), method);
    {
        let mut state = state_mutex.lock().unwrap();
        state.builder = BuilderState::Idle;
    }

    match result {
        Ok(_) => Ok("Success".to_string()),
        Err(_) => Err(())
    }
}

// Command to decompress a archive file into a directory.
#[tauri::command]
async fn decompress(window: Window, app: tauri::AppHandle, state_mutex: State<'_, Mutex<AppState>>, source_path: String, target_path:String) -> Result<String, ()> {
    {
        let mut state = state_mutex.lock().unwrap();
        state.builder = BuilderState::Running;
    }

    let result = unzip(window,app.clone(), source_path, target_path);
    {
        let mut state = state_mutex.lock().unwrap();
        state.builder = BuilderState::Idle;
    }

    match result {
        Ok(_) => Ok("Success".to_string()),
        Err(_) => Err(())
    }
}


// Comand to get the current user home
#[tauri::command]
async fn get_user_home() -> Result<String, ()> {
    match dirs::home_dir() {
        Some(path) => Ok(path.to_str().unwrap().to_string()),
        None => Err(())
    }
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![compress, decompress, get_user_home, abort_build])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
