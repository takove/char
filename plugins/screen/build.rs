const COMMANDS: &[&str] = &["ping", "capture_screenshot", "start_recording", "stop_recording", "get_recording_status"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
