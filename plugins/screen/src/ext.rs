use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResult {
    pub path: String,
    pub filename: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStatus {
    pub is_recording: bool,
    pub session_id: Option<String>,
    pub output_path: Option<String>,
}

pub struct RecordingState {
    pub is_recording: bool,
    pub session_id: Option<String>,
    pub output_path: Option<PathBuf>,
    pub process: Option<tokio::process::Child>,
}

impl Default for RecordingState {
    fn default() -> Self {
        Self {
            is_recording: false,
            session_id: None,
            output_path: None,
            process: None,
        }
    }
}

pub struct Screen<'a, R: tauri::Runtime, M: tauri::Manager<R>> {
    manager: &'a M,
    _runtime: std::marker::PhantomData<fn() -> R>,
}

impl<'a, R: tauri::Runtime, M: tauri::Manager<R>> Screen<'a, R, M> {
    pub fn ping(&self) -> Result<String, crate::Error> {
        let _ = self.manager;
        Ok("pong".to_string())
    }

    pub async fn capture_screenshot(&self, output_dir: &std::path::Path) -> Result<CaptureResult, crate::Error> {
        std::fs::create_dir_all(output_dir)
            .map_err(|e| crate::Error::Io(e.to_string()))?;

        let filename = format!("screenshot_{}.png", uuid::Uuid::new_v4());
        let output_path = output_dir.join(&filename);

        #[cfg(target_os = "macos")]
        {
            let status = tokio::process::Command::new("screencapture")
                .args(["-x", "-t", "png"])
                .arg(&output_path)
                .status()
                .await
                .map_err(|e| crate::Error::Io(e.to_string()))?;

            if !status.success() {
                return Err(crate::Error::Capture("screencapture command failed".into()));
            }
        }

        #[cfg(target_os = "linux")]
        {
            let grim_result = tokio::process::Command::new("grim")
                .arg(&output_path)
                .status()
                .await;

            match grim_result {
                Ok(status) if status.success() => {},
                _ => {
                    let scrot_result = tokio::process::Command::new("scrot")
                        .arg(&output_path)
                        .status()
                        .await;

                    match scrot_result {
                        Ok(status) if status.success() => {},
                        _ => {
                            let import_result = tokio::process::Command::new("import")
                                .args(["-window", "root"])
                                .arg(&output_path)
                                .status()
                                .await
                                .map_err(|e| crate::Error::Io(e.to_string()))?;

                            if !import_result.success() {
                                return Err(crate::Error::Capture(
                                    "No screenshot tool available (tried grim, scrot, import)".into(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            return Err(crate::Error::Capture("Windows screenshot not yet supported".into()));
        }

        Ok(CaptureResult {
            path: output_path.to_string_lossy().to_string(),
            filename,
        })
    }

    pub async fn start_recording(
        &self,
        output_dir: &std::path::Path,
        recording_state: &Arc<Mutex<RecordingState>>,
        session_id: &str,
    ) -> Result<CaptureResult, crate::Error> {
        let mut state = recording_state.lock().await;
        if state.is_recording {
            return Err(crate::Error::Capture("Already recording".into()));
        }

        std::fs::create_dir_all(output_dir)
            .map_err(|e| crate::Error::Io(e.to_string()))?;

        let filename = format!("recording_{}.mp4", uuid::Uuid::new_v4());
        let output_path = output_dir.join(&filename);

        #[cfg(target_os = "macos")]
        {
            let child = tokio::process::Command::new("screencapture")
                .args(["-v", "-t", "mp4"])
                .arg(&output_path)
                .spawn()
                .map_err(|e| crate::Error::Io(e.to_string()))?;

            state.process = Some(child);
        }

        #[cfg(target_os = "linux")]
        {
            let child = tokio::process::Command::new("ffmpeg")
                .args([
                    "-f", "x11grab",
                    "-framerate", "15",
                    "-i", ":0.0",
                    "-c:v", "libx264",
                    "-preset", "ultrafast",
                    "-crf", "28",
                ])
                .arg(&output_path)
                .spawn()
                .map_err(|e| crate::Error::Io(e.to_string()))?;

            state.process = Some(child);
        }

        state.is_recording = true;
        state.session_id = Some(session_id.to_string());
        state.output_path = Some(output_path.clone());

        Ok(CaptureResult {
            path: output_path.to_string_lossy().to_string(),
            filename,
        })
    }

    pub async fn stop_recording(
        &self,
        recording_state: &Arc<Mutex<RecordingState>>,
    ) -> Result<Option<CaptureResult>, crate::Error> {
        let mut state = recording_state.lock().await;
        if !state.is_recording {
            return Ok(None);
        }

        if let Some(ref mut process) = state.process {
            let _ = process.kill().await;
            let _ = process.wait().await;
        }

        let result = state.output_path.take().map(|path| {
            let filename = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            CaptureResult {
                path: path.to_string_lossy().to_string(),
                filename,
            }
        });

        state.is_recording = false;
        state.session_id = None;
        state.process = None;

        Ok(result)
    }

    pub async fn get_recording_status(
        &self,
        recording_state: &Arc<Mutex<RecordingState>>,
    ) -> RecordingStatus {
        let state = recording_state.lock().await;
        RecordingStatus {
            is_recording: state.is_recording,
            session_id: state.session_id.clone(),
            output_path: state.output_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        }
    }
}

pub trait ScreenPluginExt<R: tauri::Runtime> {
    fn screen(&self) -> Screen<'_, R, Self>
    where
        Self: tauri::Manager<R> + Sized;
}

impl<R: tauri::Runtime, T: tauri::Manager<R>> ScreenPluginExt<R> for T {
    fn screen(&self) -> Screen<'_, R, Self>
    where
        Self: Sized,
    {
        Screen {
            manager: self,
            _runtime: std::marker::PhantomData,
        }
    }
}
