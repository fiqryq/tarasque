mod commands;
mod models;
mod server;
mod storage;

use server::state::ServerHandle;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .manage(ServerHandle::default())
        .invoke_handler(tauri::generate_handler![
            commands::load_data,
            commands::save_data,
            commands::start_server,
            commands::stop_server,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
