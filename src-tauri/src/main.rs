#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
async fn arena_request(payload: serde_json::Value) -> Result<serde_json::Value,String> {
    tauri::async_runtime::spawn_blocking(move || {
        let request=serde_json::from_value(payload).map_err(|e|e.to_string())?;
        arena_core::bridge::handle(request)
    }).await.map_err(|e|e.to_string())?
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![arena_request])
        .run(tauri::generate_context!())
        .expect("error while running Project ARENA");
}
