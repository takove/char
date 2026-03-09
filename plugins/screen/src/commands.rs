use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ext::{CaptureResult, RecordingState, RecordingStatus};
use crate::ScreenPluginExt;

#[tauri::command]
#[specta::specta]
pub(crate) async fn ping<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<String, String> {
    app.screen().ping().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn capture_screenshot<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    output_dir: String,
) -> Result<CaptureResult, String> {
    app.screen()
        .capture_screenshot(std::path::Path::new(&output_dir))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn start_recording<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, Arc<Mutex<RecordingState>>>,
    output_dir: String,
    session_id: String,
) -> Result<CaptureResult, String> {
    app.screen()
        .start_recording(std::path::Path::new(&output_dir), &state, &session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn stop_recording<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, Arc<Mutex<RecordingState>>>,
) -> Result<Option<CaptureResult>, String> {
    app.screen()
        .stop_recording(&state)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_recording_status<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, Arc<Mutex<RecordingState>>>,
) -> Result<RecordingStatus, String> {
    Ok(app.screen().get_recording_status(&state).await)
}
