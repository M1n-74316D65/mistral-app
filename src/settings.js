// Le Chat Settings JavaScript
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

class SettingsApp {
  constructor() {
    this.newChatDefault = document.getElementById('new-chat-default');
    this.notificationsEnabled = document.getElementById('notifications-enabled');

    this.initEventListeners();
    this.loadSettings();
  }

  initEventListeners() {
    if (this.newChatDefault) {
      this.newChatDefault.addEventListener('change', () => this.saveSettings());
    }
    if (this.notificationsEnabled) {
      this.notificationsEnabled.addEventListener('change', () => this.saveSettings());
    }
  }

  async loadSettings() {
    try {
      const settings = await invoke('get_settings');
      if (this.newChatDefault) {
        this.newChatDefault.checked = settings.new_chat_default ?? true;
      }
      if (this.notificationsEnabled) {
        this.notificationsEnabled.checked = settings.notifications_enabled ?? true;
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  }

  async saveSettings() {
    const settings = {
      new_chat_default: this.newChatDefault?.checked ?? true,
      notifications_enabled: this.notificationsEnabled?.checked ?? true,
    };

    try {
      await invoke('save_settings', { settings });
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  }
}

document.addEventListener('DOMContentLoaded', () => {
  new SettingsApp();
});
