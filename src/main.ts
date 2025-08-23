import { invoke } from "@tauri-apps/api/tauri";

interface Settings {
  shortcut: string;
  translate_shortcut?: string;
  api_keys?: { [provider: string]: string };
  api_key?: string; // Legacy field for migration
  model: string;
  base_url: string;
  prompt: string;
  provider: string;
  sound_enabled?: boolean;
  notifications_enabled?: boolean;
}

window.addEventListener("DOMContentLoaded", () => {
  const settingsForm = document.getElementById("settings-form");
  const shortcutInput = document.getElementById("shortcut") as HTMLInputElement;
  const translateShortcutInput = document.getElementById("translate-shortcut") as HTMLInputElement;
  const apiKeyInput = document.getElementById("api-key") as HTMLInputElement;
  const providerSelect = document.getElementById("provider") as HTMLSelectElement;
  const modelSelect = document.getElementById("model") as HTMLSelectElement;
  const baseUrlInput = document.getElementById("base-url") as HTMLInputElement;
  const promptTextarea = document.getElementById("prompt") as HTMLTextAreaElement;
  const soundEnabledCheckbox = document.getElementById("sound-enabled") as HTMLInputElement;
  const notificationsEnabledCheckbox = document.getElementById("notifications-enabled") as HTMLInputElement;
  const saveButton = document.getElementById("save-button") as HTMLButtonElement;
  const statusDiv = document.getElementById("status") as HTMLDivElement;

  // Store API keys for each provider
  let providerApiKeys: { [provider: string]: string } = {};

  // Provider-specific configurations
  const providerConfigs = {
    openai: {
      models: [
        { value: "gpt-3.5-turbo", label: "GPT-3.5 Turbo" },
        { value: "gpt-4", label: "GPT-4" },
        { value: "gpt-4-turbo", label: "GPT-4 Turbo" },
        { value: "gpt-4o", label: "GPT-4o" }
      ],
      baseUrl: "https://api.openai.com/v1",
      apiKeyPlaceholder: "Enter your OpenAI API key"
    },
    gemini: {
      models: [
        { value: "gemini-1.5-flash", label: "Gemini 1.5 Flash" },
        { value: "gemini-1.5-pro", label: "Gemini 1.5 Pro" },
        { value: "gemini-pro", label: "Gemini Pro" }
      ],
      baseUrl: "https://generativelanguage.googleapis.com",
      apiKeyPlaceholder: "Enter your Google AI API key"
    }
  };

  async function updateProviderUI() {
    const provider = providerSelect.value as keyof typeof providerConfigs;
    const config = providerConfigs[provider];
    
    // Save current API key before switching
    const currentProvider = Object.keys(providerConfigs).find(p => 
      providerConfigs[p as keyof typeof providerConfigs].apiKeyPlaceholder === apiKeyInput.placeholder
    );
    if (currentProvider && apiKeyInput.value) {
      providerApiKeys[currentProvider] = apiKeyInput.value;
      await invoke("save_api_key_for_provider", { 
        provider: currentProvider, 
        apiKey: apiKeyInput.value 
      });
    }
    
    // Update model options
    modelSelect.innerHTML = '';
    config.models.forEach(model => {
      const option = document.createElement('option');
      option.value = model.value;
      option.textContent = model.label;
      modelSelect.appendChild(option);
    });
    
    // Update base URL if it's still default
    if (baseUrlInput.value === providerConfigs.openai.baseUrl || 
        baseUrlInput.value === providerConfigs.gemini.baseUrl || 
        baseUrlInput.value === '') {
      baseUrlInput.value = config.baseUrl;
    }
    
    // Update API key placeholder
    apiKeyInput.placeholder = config.apiKeyPlaceholder;
    
    // Load API key for new provider
    try {
      const savedApiKey = await invoke<string>("get_api_key_for_provider", { provider });
      apiKeyInput.value = savedApiKey || providerApiKeys[provider] || '';
    } catch (error) {
      console.error("Failed to load API key for provider:", error);
      apiKeyInput.value = providerApiKeys[provider] || '';
    }
  }

  // Load settings when the window opens
  invoke<Settings>("load_settings").then(async (settings) => {
    if (settings) {
      shortcutInput.value = settings.shortcut;
      translateShortcutInput.value = settings.translate_shortcut || 'CmdOrCtrl+Alt+T';
      providerSelect.value = settings.provider || 'openai';
      
      // Load API keys for all providers
      if (settings.api_keys) {
        providerApiKeys = { ...settings.api_keys };
      }
      // Handle legacy api_key field
      else if (settings.api_key) {
        providerApiKeys[settings.provider] = settings.api_key;
      }
      
      await updateProviderUI();
      modelSelect.value = settings.model;
      baseUrlInput.value = settings.base_url;
      promptTextarea.value = settings.prompt;
      soundEnabledCheckbox.checked = settings.sound_enabled !== false; // Default to true
      notificationsEnabledCheckbox.checked = settings.notifications_enabled === true; // Default to false
    } else {
      await updateProviderUI();
      // Set defaults for new installations
      soundEnabledCheckbox.checked = true;
      notificationsEnabledCheckbox.checked = false;
    }
  });

  // Update UI when provider changes
  providerSelect.addEventListener('change', updateProviderUI);

  function showStatus(message: string, isError: boolean = false) {
    statusDiv.textContent = message;
    statusDiv.className = `status ${isError ? 'error' : 'success'}`;
    statusDiv.style.display = 'block';
    setTimeout(() => {
      statusDiv.style.display = 'none';
    }, 3000);
  }

  settingsForm?.addEventListener("submit", async (e) => {
    e.preventDefault();
    saveButton.disabled = true;
    saveButton.textContent = "Saving...";

    try {
      // Save current API key for current provider
      const currentProvider = providerSelect.value;
      if (apiKeyInput.value) {
        providerApiKeys[currentProvider] = apiKeyInput.value;
        await invoke("save_api_key_for_provider", { 
          provider: currentProvider, 
          apiKey: apiKeyInput.value 
        });
      }

      const settings: Settings = {
        shortcut: shortcutInput.value,
        translate_shortcut: translateShortcutInput.value,
        api_keys: providerApiKeys,
        provider: providerSelect.value,
        model: modelSelect.value,
        base_url: baseUrlInput.value,
        prompt: promptTextarea.value,
        sound_enabled: soundEnabledCheckbox.checked,
        notifications_enabled: notificationsEnabledCheckbox.checked,
      };

      await invoke("save_settings", { settings });
      showStatus("Settings saved successfully!");
    } catch (error) {
      showStatus(`Failed to save settings: ${error}`, true);
    } finally {
      saveButton.disabled = false;
      saveButton.textContent = "Save Settings";
    }
  });
});