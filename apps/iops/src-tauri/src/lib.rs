use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn open_main_window(app: tauri::AppHandle) -> Result<(), String> {
    // Tạo main window
    // let main_window = WebviewWindowBuilder::new(
    //     &app,
    //     "main",
    //     WebviewUrl::App("/#/dashboard".into()),
    // )
    // .build()
    // .map_err(|e| e.to_string())?;

    // main_window.show().map_err(|e| e.to_string())?;

    if let Some(main_win) = app.get_webview_window("main") {
        main_win.show().map_err(|e| e.to_string())?;
    }

    // Đóng login window
    if let Some(login_win) = app.get_webview_window("login") {
        login_win.close().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, open_main_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
