#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
async fn dispatch(request: serde_json::Value) -> Result<kitcontrol_core::Response, String> {
    tauri::async_runtime::spawn_blocking(move || {
        kitcontrol_core::dispatch_json(&request.to_string())
    })
    .await
    .map_err(|error| format!("Core worker failed: {error}"))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![dispatch])
        .run(tauri::generate_context!())
        .expect("Failed to start KitControl desktop runtime");
}
