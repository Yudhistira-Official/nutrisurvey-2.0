pub mod commands {
    #[tauri::command]
    pub fn ping() -> &'static str {
        "pong"
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![commands::ping])
        .run(tauri::generate_context!())
        .expect("error while running NutriSurvey");
}

#[cfg(test)]
mod tests {
    #[test]
    fn ping_returns_pong() {
        assert_eq!(crate::commands::ping(), "pong");
    }
}
