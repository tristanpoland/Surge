use tauri::{Manager, WebviewWindow};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tauri_plugin_positioner::{Position, WindowExt};
use window_vibrancy::{apply_blur, apply_vibrancy, NSVisualEffectMaterial};

#[tauri::command]
async fn toggle_window(window: WebviewWindow) -> Result<(), String> {
    if window.is_visible().unwrap_or(false) {
        window.hide().map_err(|e| e.to_string())?;
    } else {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        window.move_window(Position::Center).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                let window = app.get_webview_window("main").unwrap();
                
                #[cfg(target_os = "macos")]
                apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
                    .expect("Failed to apply vibrancy");
                
                #[cfg(target_os = "windows")]
                apply_blur(&window, Some((18, 18, 18, 125)))
                    .expect("Failed to apply blur");
        
                let shortcut = if cfg!(target_os = "macos") {
                    Shortcut::new(Some(Modifiers::SUPER), Code::Space)
                } else {
                    Shortcut::new(Some(Modifiers::ALT), Code::Space)
                };
        
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |_app_handle, _window, _shortcut| {
                            let window = window.clone();
                            if !window.is_visible().unwrap_or(false) {
                                tauri::async_runtime::spawn(async move {
                                    let _ = toggle_window(window).await;
                                });
                            }
                        })
                        .build(),
                )?;
                app.global_shortcut().register(shortcut)?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![toggle_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}