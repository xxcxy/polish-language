// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use get_selected_text::get_selected_text;
use tauri::{GlobalShortcutManager, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, ClipboardManager};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicyAccessory};



#[derive(Serialize, Deserialize, Clone)]
struct Settings {
    shortcut: String,
    api_key: String,
    model: String,
    base_url: String,
    prompt: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            shortcut: "CmdOrCtrl+Shift+P".to_string(),
            api_key: "".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            prompt: "Please polish and improve the following text while maintaining its original meaning and tone:".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Serialize, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

fn get_settings_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("polish-language");
    fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

#[tauri::command]
fn save_settings(settings: Settings) -> Result<(), String> {
    let settings_path = get_settings_path();
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    fs::write(settings_path, json)
        .map_err(|e| format!("Failed to write settings: {}", e))?;
    
    Ok(())
}

#[tauri::command]
fn load_settings() -> Settings {
    let settings_path = get_settings_path();
    
    if let Ok(content) = fs::read_to_string(settings_path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Settings::default()
    }
}

async fn polish_text_with_llm(text: &str, settings: &Settings) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let request = OpenAIRequest {
        model: settings.model.clone(),
        messages: vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: settings.prompt.clone(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: text.to_string(),
            },
        ],
        max_tokens: 1000,
        temperature: 0.3,
    };

    let response = client
        .post(&format!("{}/chat/completions", settings.base_url))
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()));
    }

    let openai_response: OpenAIResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    openai_response
        .choices
        .first()
        .map(|choice| choice.message.content.trim().to_string())
        .ok_or_else(|| "No response from API".to_string())
}

#[tokio::main]
async fn main() {
    let tray_menu = SystemTrayMenu::new()
        .add_item(tauri::CustomMenuItem::new("settings".to_string(), "Settings"))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(tauri::CustomMenuItem::new("quit".to_string(), "Quit"));
    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![save_settings, load_settings])
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "settings" => {
                    if let Some(window) = app.get_window("settings") {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    } else {
                        tauri::WindowBuilder::new(
                            app,
                            "settings",
                            tauri::WindowUrl::App("index.html".into()),
                        )
                        .title("Polish Language - Settings")
                        .inner_size(500.0, 600.0)
                        .resizable(false)
                        .build()
                        .unwrap();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .setup(|app| {
            // Hide dock icon on macOS
            #[cfg(target_os = "macos")]
            unsafe {
                NSApp().setActivationPolicy_(NSApplicationActivationPolicyAccessory);
            }
            
            let app_handle = app.handle();
            let settings = load_settings();
            
            // Register the global shortcut
            let shortcut = settings.shortcut.clone();
            app.global_shortcut_manager()
                .register(&shortcut, move || {
                    let app_handle_clone = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let selected_text = match get_selected_text() {
                            Ok(text) => text,
                            Err(e) => {
                                eprintln!("Error getting selected text: {:?}", e);
                                return;
                            }
                        };
                        
                        if selected_text.trim().is_empty() {
                            return;
                        }
                        
                        let settings = load_settings();
                        if settings.api_key.is_empty() {
                            eprintln!("API key not configured");
                            return;
                        }
                        
                        match polish_text_with_llm(&selected_text, &settings).await {
                            Ok(polished_text) => {
                                // Try to replace selected text or copy to clipboard
                                if let Err(_) = app_handle_clone.clipboard_manager().write_text(polished_text) {
                                    eprintln!("Failed to write to clipboard");
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to polish text: {}", e);
                            }
                        }
                    });
                })
                .unwrap_or_else(|e| eprintln!("Failed to register shortcut: {}", e));
            
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
