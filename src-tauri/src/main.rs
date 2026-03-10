mod commands;
mod models;
mod proxy;
mod server;
mod storage;

use server::state::MultiServerHandle;

fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .manage(MultiServerHandle::default())
        .invoke_handler(tauri::generate_handler![
            commands::load_data,
            commands::save_data,
            commands::start_server,
            commands::stop_server,
            commands::get_running_collections,
            commands::find_free_port,
            commands::import_openapi,
            commands::export_collection_json,
            commands::export_collection_postman,
            commands::run_test_suite,
            commands::generate_alias_cert,
            commands::trust_cert_macos,
            commands::add_hosts_entry,
            commands::write_cert_files,
            commands::start_proxy,
            commands::stop_proxy,
            commands::get_proxy_status,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
