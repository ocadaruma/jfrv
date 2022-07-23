#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::io::{BufReader, Read};
use std::time::SystemTime;

mod jfr;

#[tauri::command]
fn jfr_file() -> String {
  println!("{}: start read", unix_time());
  let path = std::env::var("JFR_FILE_PATH").unwrap();
  let mut buf = String::new();
  println!("{}: before read", unix_time());
  let mut reader = BufReader::new(std::fs::File::open(path).unwrap());
  reader.read_to_string(&mut buf).unwrap();
  println!("{}: end read", unix_time());
  buf
}

fn main() {
  let http = tauri_invoke_http::Invoke::new(if cfg!(feature = "custom-protocol") {
    ["tauri://localhost"]
  } else {
    ["http://localhost:8080"]
  });

  tauri::Builder::default()
      .invoke_system(http.initialization_script(), http.responder())
      .setup(move |app| { http.start(app.handle()); Ok(()) })
      .invoke_handler(tauri::generate_handler![jfr_file])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}

fn unix_time() -> u64 {
  std::time::SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64
}
