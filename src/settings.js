// Le Chat Settings JavaScript
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

class SettingsApp {
  constructor() {
    this.newChatDefault = document.getElementById('new-chat-default');
    this.notificationsEnabled = document.getElementById('notifications-enabled');
    this.shortcutKey = document.getElementById('shortcut-key');
    this.clearHistoryBtn = document.getElementById('clear-history-btn');

    this.initEventListeners();
    this.loadSettings();
    this.initPlatformDetails();
  }

  initPlatformDetails() {
    // Detect Mac
    const isMac = navigator.platform?.toUpperCase().includes('MAC')
      || navigator.userAgent?.includes('Mac');
    
    if (this.shortcutKey) {
      this.shortcutKey.textContent = isMac ? '⌥ + Space' : 'Alt + Space';
    }
  }

  initEventListeners() {
    if (this.newChatDefault) {
      this.newChatDefault.addEventListener('change', () => this.saveSettings());
    }
    if (this.notificationsEnabled) {
      this.notificationsEnabled.addEventListener('change', () => this.saveSettings());
    }
    if (this.clearHistoryBtn) {
      this.clearHistoryBtn.addEventListener('click', () => this.clearHistory());
    }

    // Listen for theme-changed event from the main window directly
    listen('theme-changed', (event) => {
      const { theme } = event.payload || {};
      if (theme) {
        this.applyTheme(theme);
      }
    }).catch(error => {
      console.error('Failed to listen for theme-changed event:', error);
    });

    // Listen for settings-changed event
    listen('settings-changed', (event) => {
      const settings = event.payload;
      if (settings && settings.theme) {
        this.applyTheme(settings.theme);
      }
    }).catch(error => {
      console.error('Failed to listen for settings-changed event:', error);
    });
  }

  applyTheme(theme) {
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
      document.documentElement.classList.remove('light');
    } else if (theme === 'light') {
      document.documentElement.classList.add('light');
      document.documentElement.classList.remove('dark');
    }
  }

  clearHistory() {
    localStorage.removeItem('launcher_history');
    
    if (this.clearHistoryBtn) {
      const originalText = this.clearHistoryBtn.textContent;
      this.clearHistoryBtn.textContent = 'Cleared!';
      this.clearHistoryBtn.classList.add('success');
      this.clearHistoryBtn.disabled = true;
      
      setTimeout(() => {
        this.clearHistoryBtn.textContent = originalText;
        this.clearHistoryBtn.classList.remove('success');
        this.clearHistoryBtn.disabled = false;
      }, 2000);
    }
  }

  async loadSettings() {
    try {
      const settings = await invoke('get_settings');
      this.currentTheme = settings.theme; // store the current theme
      if (settings.theme) {
        this.applyTheme(settings.theme);
      }
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
      theme: this.currentTheme, // Preserve the theme that was synced from Mistral
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
