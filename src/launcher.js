// Le Chat Launcher JavaScript
const { invoke } = window.__TAURI__.core;

const input = document.getElementById('launcher-input');
const submitBtn = document.getElementById('submit-btn');

// Focus input on load
window.addEventListener('DOMContentLoaded', () => {
  input.focus();
});

// Handle keyboard events
document.addEventListener('keydown', async (e) => {
  // Escape to hide launcher
  if (e.key === 'Escape') {
    e.preventDefault();
    await invoke('hide_launcher');
    return;
  }
  
  // Enter to submit
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    await submitMessage();
    return;
  }
});

// Submit button click
submitBtn.addEventListener('click', async () => {
  await submitMessage();
});

// Submit the message
async function submitMessage() {
  const message = input.value.trim();
  
  if (!message) {
    return;
  }
  
  try {
    // Clear input
    input.value = '';
    
    // Send message to Rust backend
    await invoke('submit_message', { message });
  } catch (error) {
    console.error('Failed to submit message:', error);
    // Restore message on error
    input.value = message;
  }
}

// Re-focus input when window gains focus
window.addEventListener('focus', () => {
  input.focus();
  input.select();
});
