mod commands;
mod db;
mod error;
mod files;
mod keychain;
mod state;
mod tray;

use state::AppState;
use tauri::Manager;

pub fn run() {
    let tray_only = std::env::args().any(|a| a == "--tray");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Un second lancement ramène la fenêtre plutôt que d'ouvrir une autre instance.
            tray::show_main(app);
        }))
        .setup(move |app| {
            let app_data = app.path().app_data_dir()?;
            std::fs::create_dir_all(app_data.join("files").join("uploads"))?;
            let conn = db::open(&app_data).map_err(|e| e.to_string())?;
            app.manage(AppState {
                db: std::sync::Mutex::new(conn),
                app_data,
            });
            tray::build_tray(app.handle())?;
            if tray_only {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Fermer la fenêtre = masquer (l'app reste joignable via le tray, sans surcoût).
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::set_front_view,
            commands::support_abs_path,
            commands::list_folders,
            commands::create_folder,
            commands::rename_folder,
            commands::move_folder,
            commands::delete_folder,
            commands::list_docs,
            commands::get_doc,
            commands::create_doc,
            commands::rename_doc,
            commands::move_doc,
            commands::set_doc_flags,
            commands::set_exam_date,
            commands::delete_doc,
            commands::import_paths,
            commands::delete_support,
            commands::merge_docs,
            commands::split_support,
            commands::get_settings,
            commands::set_settings,
            commands::set_api_key,
            commands::has_api_key,
            commands::delete_api_key,
            commands::list_models,
            commands::hide_window,
            commands::show_window
        ])
        .run(tauri::generate_context!())
        .expect("erreur WLarp");
}
