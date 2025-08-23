// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use get_selected_text::get_selected_text;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{
    api::notification::Notification, ClipboardManager, GlobalShortcutManager, Manager, SystemTray,
    SystemTrayEvent, SystemTrayMenu,
};

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicyAccessory};

#[derive(Serialize, Deserialize, Clone)]
struct Settings {
    shortcut: String,
    #[serde(default = "default_translate_shortcut")]
    translate_shortcut: String,
    #[serde(default)]
    api_keys: HashMap<String, String>, // provider -> api_key mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<String>, // Legacy field for migration
    model: String,
    base_url: String,
    prompt: String,
    provider: String,
    #[serde(default = "default_sound_enabled")]
    sound_enabled: bool,
    #[serde(default = "default_notifications_enabled")]
    notifications_enabled: bool,
}

fn default_sound_enabled() -> bool {
    true
}

fn default_notifications_enabled() -> bool {
    false
}

fn default_translate_shortcut() -> String {
    "CmdOrCtrl+Alt+T".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            shortcut: "CmdOrCtrl+Alt+P".to_string(),
            translate_shortcut: default_translate_shortcut(),
            api_keys: HashMap::new(),
            api_key: None,
            model: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            prompt: "Please polish and improve the following text while maintaining its original meaning and tone:".to_string(),
            provider: "openai".to_string(),
            sound_enabled: default_sound_enabled(),
            notifications_enabled: default_notifications_enabled(),
        }
    }
}

impl Settings {
    fn get_current_api_key(&self) -> String {
        self.api_keys
            .get(&self.provider)
            .cloned()
            .unwrap_or_default()
    }

    fn set_api_key(&mut self, provider: &str, api_key: &str) {
        if api_key.is_empty() {
            self.api_keys.remove(provider);
        } else {
            self.api_keys
                .insert(provider.to_string(), api_key.to_string());
        }
    }

    // Migration helper to convert old single api_key to provider-based keys
    fn migrate_legacy_api_key(&mut self) {
        if let Some(legacy_key) = &self.api_key {
            if !legacy_key.is_empty() && !self.api_keys.contains_key(&self.provider) {
                self.api_keys
                    .insert(self.provider.clone(), legacy_key.clone());
            }
            self.api_key = None; // Clear legacy field after migration
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

// Gemini API structures
#[derive(Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize, Deserialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Serialize, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

fn get_settings_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("polish-language");
    fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

fn play_completion_sound() {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("afplay")
            .arg("/System/Library/Sounds/Glass.aiff")
            .spawn();
    }
}

fn show_notification(app_handle: &tauri::AppHandle, title: &str, body: &str, settings: &Settings) {
    if settings.notifications_enabled {
        let _ = Notification::new(&app_handle.config().tauri.bundle.identifier)
            .title(title)
            .body(body)
            .show();
    }
}

fn update_tray_icon_processing(app_handle: &tauri::AppHandle, processing: bool) {
    let tray = app_handle.tray_handle();
    // On macOS, we can change the tray icon to indicate processing
    // For now, we'll just use the tooltip to show status
    let tooltip = if processing {
        "Polish Language - Processing..."
    } else {
        "Polish Language"
    };
    let _ = tray.set_tooltip(tooltip);
}

#[tauri::command]
fn save_settings(mut settings: Settings) -> Result<(), String> {
    // Ensure legacy field is cleared
    settings.api_key = None;

    let settings_path = get_settings_path();
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(settings_path, json).map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

#[tauri::command]
fn get_api_key_for_provider(provider: String) -> String {
    let settings = load_settings();
    settings
        .api_keys
        .get(&provider)
        .cloned()
        .unwrap_or_default()
}

#[tauri::command]
fn save_api_key_for_provider(provider: String, api_key: String) -> Result<(), String> {
    let mut settings = load_settings();
    settings.set_api_key(&provider, &api_key);
    save_settings(settings)
}

#[tauri::command]
fn load_settings() -> Settings {
    let settings_path = get_settings_path();

    if let Ok(content) = fs::read_to_string(settings_path) {
        let mut settings: Settings = serde_json::from_str(&content).unwrap_or_default();
        settings.migrate_legacy_api_key();
        settings
    } else {
        Settings::default()
    }
}

async fn polish_text_with_llm(text: &str, settings: &Settings) -> Result<String, String> {
    let client = reqwest::Client::new();

    match settings.provider.as_str() {
        "gemini" => polish_text_with_gemini(text, settings, &client).await,
        _ => polish_text_with_openai(text, settings, &client).await,
    }
}

async fn translate_text_with_llm(text: &str, settings: &Settings) -> Result<String, String> {
    let client = reqwest::Client::new();

    let translate_prompt = "Translate the following text to English. If the text is already in English, keep it as is. Only return the translated text without any additional explanation:";

    match settings.provider.as_str() {
        "gemini" => translate_text_with_gemini(text, translate_prompt, settings, &client).await,
        _ => translate_text_with_openai(text, translate_prompt, settings, &client).await,
    }
}

async fn polish_text_with_openai(
    text: &str,
    settings: &Settings,
    client: &reqwest::Client,
) -> Result<String, String> {
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
        .post(format!("{}/chat/completions", settings.base_url))
        .header(
            "Authorization",
            format!("Bearer {}", settings.get_current_api_key()),
        )
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "API request failed with status: {}",
            response.status()
        ));
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

async fn polish_text_with_gemini(
    text: &str,
    settings: &Settings,
    client: &reqwest::Client,
) -> Result<String, String> {
    let combined_prompt = format!("{}\n\n{}", settings.prompt, text);

    let request = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart {
                text: combined_prompt,
            }],
        }],
        generation_config: GeminiGenerationConfig {
            temperature: 0.3,
            max_output_tokens: 1000,
        },
    };

    let api_key = settings.get_current_api_key();
    let url = if settings.base_url.contains("generateContent") {
        format!("{}?key={}", settings.base_url, api_key)
    } else {
        format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            settings.base_url, settings.model, api_key
        )
    };

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!(
            "API request failed with status: {} - {}",
            status, error_text
        ));
    }

    let gemini_response: GeminiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    gemini_response
        .candidates
        .first()
        .and_then(|candidate| candidate.content.parts.first())
        .map(|part| part.text.trim().to_string())
        .ok_or_else(|| "No response from API".to_string())
}

async fn translate_text_with_openai(
    text: &str,
    translate_prompt: &str,
    settings: &Settings,
    client: &reqwest::Client,
) -> Result<String, String> {
    let request = OpenAIRequest {
        model: settings.model.clone(),
        messages: vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: translate_prompt.to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: text.to_string(),
            },
        ],
        max_tokens: 1000,
        temperature: 0.1, // Lower temperature for more consistent translations
    };

    let response = client
        .post(format!("{}/chat/completions", settings.base_url))
        .header(
            "Authorization",
            format!("Bearer {}", settings.get_current_api_key()),
        )
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "API request failed with status: {}",
            response.status()
        ));
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

async fn translate_text_with_gemini(
    text: &str,
    translate_prompt: &str,
    settings: &Settings,
    client: &reqwest::Client,
) -> Result<String, String> {
    let combined_prompt = format!("{}\n\n{}", translate_prompt, text);

    let request = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart {
                text: combined_prompt,
            }],
        }],
        generation_config: GeminiGenerationConfig {
            temperature: 0.1, // Lower temperature for more consistent translations
            max_output_tokens: 1000,
        },
    };

    let api_key = settings.get_current_api_key();
    let url = if settings.base_url.contains("generateContent") {
        format!("{}?key={}", settings.base_url, api_key)
    } else {
        format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            settings.base_url, settings.model, api_key
        )
    };

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!(
            "API request failed with status: {} - {}",
            status, error_text
        ));
    }

    let gemini_response: GeminiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    gemini_response
        .candidates
        .first()
        .and_then(|candidate| candidate.content.parts.first())
        .map(|part| part.text.trim().to_string())
        .ok_or_else(|| "No response from API".to_string())
}

#[tokio::main]
async fn main() {
    let tray_menu = SystemTrayMenu::new()
        .add_item(tauri::CustomMenuItem::new(
            "settings".to_string(),
            "Settings",
        ))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(tauri::CustomMenuItem::new("quit".to_string(), "Quit"));
    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_settings,
            load_settings,
            get_api_key_for_provider,
            save_api_key_for_provider
        ])
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                match id.as_str() {
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
                }
            }
        })
        .setup(|app| {
            // Hide dock icon on macOS
            #[cfg(target_os = "macos")]
            unsafe {
                NSApp().setActivationPolicy_(NSApplicationActivationPolicyAccessory);
            }

            let app_handle = app.handle();
            let settings = load_settings();

            // Register the polish text global shortcut
            let polish_shortcut = settings.shortcut.clone();
            let app_handle_polish = app_handle.clone();
            app.global_shortcut_manager()
                .register(&polish_shortcut, move || {
                    let app_handle_clone = app_handle_polish.clone();
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
                        if settings.get_current_api_key().is_empty() {
                            eprintln!("API key not configured for provider: {}", settings.provider);
                            return;
                        }

                        // Show processing state
                        update_tray_icon_processing(&app_handle_clone, true);

                        match polish_text_with_llm(&selected_text, &settings).await {
                            Ok(polished_text) => {
                                // Copy to clipboard
                                if app_handle_clone
                                    .clipboard_manager()
                                    .write_text(polished_text.clone())
                                    .is_err()
                                {
                                    eprintln!("Failed to write to clipboard");
                                }

                                // Show completion feedback
                                if settings.sound_enabled {
                                    play_completion_sound();
                                }

                                let preview = if polished_text.len() > 100 {
                                    format!("{}...", &polished_text[..97])
                                } else {
                                    polished_text
                                };

                                show_notification(
                                    &app_handle_clone,
                                    "Text Polished",
                                    &format!("Polished text copied to clipboard:\n{}", preview),
                                    &settings,
                                );
                            }
                            Err(e) => {
                                eprintln!("Failed to polish text: {}", e);
                                show_notification(
                                    &app_handle_clone,
                                    "Polish Failed",
                                    &format!("Failed to polish text: {}", e),
                                    &settings,
                                );
                            }
                        }

                        // Reset processing state
                        update_tray_icon_processing(&app_handle_clone, false);
                    });
                })
                .unwrap_or_else(|e| eprintln!("Failed to register polish shortcut: {}", e));

            // Register the translate text global shortcut
            let translate_shortcut = settings.translate_shortcut.clone();
            let app_handle_translate = app_handle.clone();
            app.global_shortcut_manager()
                .register(&translate_shortcut, move || {
                    let app_handle_clone = app_handle_translate.clone();
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
                        if settings.get_current_api_key().is_empty() {
                            eprintln!("API key not configured for provider: {}", settings.provider);
                            return;
                        }

                        // Show processing state
                        update_tray_icon_processing(&app_handle_clone, true);

                        match translate_text_with_llm(&selected_text, &settings).await {
                            Ok(translated_text) => {
                                // Copy to clipboard
                                if app_handle_clone
                                    .clipboard_manager()
                                    .write_text(translated_text.clone())
                                    .is_err()
                                {
                                    eprintln!("Failed to write to clipboard");
                                }

                                // Show completion feedback
                                if settings.sound_enabled {
                                    play_completion_sound();
                                }

                                let preview = if translated_text.len() > 100 {
                                    format!("{}...", &translated_text[..97])
                                } else {
                                    translated_text
                                };

                                show_notification(
                                    &app_handle_clone,
                                    "Text Translated",
                                    &format!("Translated text copied to clipboard:\n{}", preview),
                                    &settings,
                                );
                            }
                            Err(e) => {
                                eprintln!("Failed to translate text: {}", e);
                                show_notification(
                                    &app_handle_clone,
                                    "Translation Failed",
                                    &format!("Failed to translate text: {}", e),
                                    &settings,
                                );
                            }
                        }

                        // Reset processing state
                        update_tray_icon_processing(&app_handle_clone, false);
                    });
                })
                .unwrap_or_else(|e| eprintln!("Failed to register translate shortcut: {}", e));
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
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
