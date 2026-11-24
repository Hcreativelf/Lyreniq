#[tauri::command]
fn runtime_label() -> String {
    format!(
        "Lyreniq {} · {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    )
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![runtime_label])
        .run(tauri::generate_context!())
        .expect("failed to run Lyreniq desktop application");
}
