use std::{collections::HashMap, sync::Arc, time::Duration};
use fuzzy_matcher::FuzzyMatcher;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use tauri::{Manager, WebviewWindow, async_runtime::spawn};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tauri_plugin_positioner::{Position, WindowExt};
use tokio::time::sleep;
use walkdir::WalkDir;
use window_vibrancy::{apply_blur, apply_vibrancy, NSVisualEffectMaterial};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexEntry {
    path: String,
    name: String,
    metadata: HashMap<String, String>,
}

pub struct SystemIndex {
    entries: Arc<RwLock<Vec<IndexEntry>>>,
    matcher: fuzzy_matcher::skim::SkimMatcherV2,
}

impl SystemIndex {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            matcher: fuzzy_matcher::skim::SkimMatcherV2::default(),
        }
    }

    pub async fn start_indexing(index: Arc<Self>) {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        
        spawn(async move {
            loop {
                interval.tick().await;
                let mut batch = Vec::with_capacity(1000);
                
                #[cfg(target_os = "windows")]
                let root = "C:\\";
                #[cfg(target_os = "macos")]
                let root = "/";
                
                for entry in WalkDir::new(root)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| !e.file_type().is_dir()) {
                    
                    if batch.len() >= 1000 {
                        let new_entries = batch.drain(..).map(|path: walkdir::DirEntry| IndexEntry {
                            name: path.file_name().to_string_lossy().into_owned(),
                            path: path.path().to_string_lossy().into_owned(),
                            metadata: HashMap::new(),
                        }).collect::<Vec<_>>();
                        
                        index.entries.write().extend(new_entries);
                        sleep(Duration::from_millis(100)).await;
                    }
                    
                    batch.push(entry);
                }
                
                // Process remaining entries
                if !batch.is_empty() {
                    let new_entries = batch.drain(..).map(|path| IndexEntry {
                        name: path.file_name().to_string_lossy().into_owned(),
                        path: path.path().to_string_lossy().into_owned(),
                        metadata: HashMap::new(),
                    }).collect::<Vec<_>>();
                    
                    index.entries.write().extend(new_entries);
                }
            }
        });
    }

    pub fn search(&self, query: &str) -> Vec<IndexEntry> {
        let entries = self.entries.read();
        let mut matches: Vec<_> = entries
            .iter()
            .filter_map(|entry| {
                self.matcher
                    .fuzzy_match(&entry.name, query)
                    .map(|score| (score, entry.clone()))
            })
            .collect();
        
        matches.sort_by(|a, b| b.0.cmp(&a.0));
        matches.into_iter().map(|(_, entry)| entry).collect()
    }
}

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

#[tauri::command]
async fn search_index(
    state: tauri::State<'_, Arc<SystemIndex>>,
    query: String
) -> Result<Vec<IndexEntry>, String> {
    Ok(state.search(&query))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let index = Arc::new(SystemIndex::new());
    let index_clone = index.clone();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
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
                
                tauri::async_runtime::spawn(async move {
                    SystemIndex::start_indexing(index_clone).await;
                });
            }
            Ok(())
        })
        .manage(index)
        .invoke_handler(tauri::generate_handler![toggle_window, search_index])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}