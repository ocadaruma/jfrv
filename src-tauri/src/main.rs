#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod jfr;

#[tauri::command]
fn jfr_file_path() -> String {
  std::env::var("JFR_FILE_PATH").unwrap()
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![jfr_file_path])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
