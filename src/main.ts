import { invoke } from "@tauri-apps/api/tauri";

interface Settings {
  shortcut: string;
  api_key: string;
  model: string;
  base_url: string;
  prompt: string;
}

window.addEventListener("DOMContentLoaded", () => {
  const settingsForm = document.getElementById("settings-form");
  const shortcutInput = document.getElementById("shortcut") as HTMLInputElement;
  const apiKeyInput = document.getElementById("api-key") as HTMLInputElement;
  const modelSelect = document.getElementById("model") as HTMLSelectElement;
  const baseUrlInput = document.getElementById("base-url") as HTMLInputElement;
  const promptTextarea = document.getElementById("prompt") as HTMLTextAreaElement;
  const saveButton = document.getElementById("save-button") as HTMLButtonElement;
  const statusDiv = document.getElementById("status") as HTMLDivElement;

  // Load settings when the window opens
  invoke<Settings>("load_settings").then((settings) => {
    if (settings) {
      shortcutInput.value = settings.shortcut;
      apiKeyInput.value = settings.api_key;
      modelSelect.value = settings.model;
      baseUrlInput.value = settings.base_url;
      promptTextarea.value = settings.prompt;
    }
  });

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

    const settings: Settings = {
      shortcut: shortcutInput.value,
      api_key: apiKeyInput.value,
      model: modelSelect.value,
      base_url: baseUrlInput.value,
      prompt: promptTextarea.value,
    };

    try {
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