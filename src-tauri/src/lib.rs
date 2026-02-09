use tauri::{
    AppHandle, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

// JavaScript to inject message into Mistral's chat input with retry logic
fn get_inject_message_js(message: &str) -> String {
    let escaped_message = message
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('$', "\\$")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    
    format!(r#"
        (function() {{
            const message = `{}`;
            const maxRetries = 10;
            const retryDelay = 300;
            let retryCount = 0;
            
            function injectMessage() {{
                // Try multiple selectors for the textarea
                const textarea = document.querySelector('textarea[placeholder*="Ask"]')
                    || document.querySelector('textarea[placeholder*="Message"]')
                    || document.querySelector('textarea[data-testid]')
                    || document.querySelector('div[contenteditable="true"]')
                    || document.querySelector('textarea');
                
                if (!textarea) {{
                    retryCount++;
                    if (retryCount < maxRetries) {{
                        console.log('[Le Chat] Waiting for textarea... attempt', retryCount);
                        setTimeout(injectMessage, retryDelay);
                        return;
                    }}
                    console.error('[Le Chat] Could not find textarea after', maxRetries, 'attempts');
                    return false;
                }}
                
                console.log('[Le Chat] Found textarea:', textarea);
                
                // Handle contenteditable div (common in modern chat UIs)
                if (textarea.contentEditable === 'true') {{
                    textarea.innerHTML = message;
                    textarea.focus();
                    textarea.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    setTimeout(submitForm, 200);
                    return true;
                }}
                
                // Use native setter to properly update React state
                const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
                    window.HTMLTextAreaElement.prototype, 
                    'value'
                ).set;
                
                nativeInputValueSetter.call(textarea, message);
                
                // Dispatch events to notify React
                textarea.dispatchEvent(new Event('input', {{ bubbles: true }}));
                textarea.dispatchEvent(new Event('change', {{ bubbles: true }}));
                
                // Focus the textarea
                textarea.focus();
                
                // Submit after input is set
                setTimeout(submitForm, 200);
                return true;
            }}
            
            function submitForm() {{
                // Look for send button - try multiple selectors
                const sendBtn = document.querySelector('button[type="submit"]')
                    || document.querySelector('button[aria-label*="send" i]')
                    || document.querySelector('button[aria-label*="Send" i]')
                    || document.querySelector('button[data-testid*="send" i]')
                    || document.querySelector('form button:last-of-type')
                    || document.querySelector('button svg[class*="send" i]')?.closest('button');
                
                if (sendBtn && !sendBtn.disabled) {{
                    console.log('[Le Chat] Clicking send button');
                    sendBtn.click();
                }} else {{
                    // If no button found, try pressing Enter
                    console.log('[Le Chat] No send button, trying Enter key');
                    const textarea = document.querySelector('textarea') 
                        || document.querySelector('div[contenteditable="true"]');
                    if (textarea) {{
                        textarea.dispatchEvent(new KeyboardEvent('keydown', {{
                            key: 'Enter',
                            code: 'Enter',
                            keyCode: 13,
                            which: 13,
                            bubbles: true,
                            cancelable: true
                        }}));
                    }}
                }}
            }}
            
            // Start the injection process
            injectMessage();
        }})();
    "#, escaped_message)
}

#[tauri::command]
async fn hide_launcher(app: AppHandle) -> Result<(), String> {
    if let Some(launcher) = app.get_webview_window("launcher") {
        launcher.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn show_launcher(app: AppHandle) -> Result<(), String> {
    if let Some(launcher) = app.get_webview_window("launcher") {
        launcher.center().map_err(|e| e.to_string())?;
        launcher.show().map_err(|e| e.to_string())?;
        launcher.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_launcher(app: AppHandle) -> Result<(), String> {
    if let Some(launcher) = app.get_webview_window("launcher") {
        let is_visible = launcher.is_visible().unwrap_or(false);
        if is_visible {
            launcher.hide().map_err(|e| e.to_string())?;
        } else {
            launcher.center().map_err(|e| e.to_string())?;
            launcher.show().map_err(|e| e.to_string())?;
            launcher.set_focus().map_err(|e| e.to_string())?;
            // Emit event to clear and focus input
            let _ = launcher.emit("launcher-shown", ());
        }
    }
    Ok(())
}

#[tauri::command]
async fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(main_window) = app.get_webview_window("main") {
        main_window.show().map_err(|e| e.to_string())?;
        main_window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn submit_message(app: AppHandle, message: String) -> Result<(), String> {
    // Hide the launcher first
    if let Some(launcher) = app.get_webview_window("launcher") {
        launcher.hide().map_err(|e| e.to_string())?;
    }
    
    // Show and focus main window
    if let Some(main_window) = app.get_webview_window("main") {
        main_window.show().map_err(|e| e.to_string())?;
        main_window.set_focus().map_err(|e| e.to_string())?;
        
        // Wait a bit for the window to be ready and page to be interactive
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        // Inject the message
        let js = get_inject_message_js(&message);
        main_window.eval(&js).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Show Le Chat", true, None::<&str>)?;
    let launcher_item = MenuItem::with_id(app, "launcher", "Quick Ask...", true, None::<&str>)?;
    let separator = MenuItem::with_id(app, "sep", "─────────", false, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[&show_item, &launcher_item, &separator, &quit_item])?;
    
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "launcher" => {
                    if let Some(launcher) = app.get_webview_window("launcher") {
                        let _ = launcher.center();
                        let _ = launcher.show();
                        let _ = launcher.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;
    
    Ok(())
}

fn setup_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Platform-specific shortcut: Alt+Space on Windows, Option+Space on macOS
    #[cfg(target_os = "macos")]
    let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
    
    #[cfg(target_os = "windows")]
    let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
    
    #[cfg(target_os = "linux")]
    let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
    
    let app_handle = app.clone();
    
    app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            if let Some(launcher) = app_handle.get_webview_window("launcher") {
                let is_visible = launcher.is_visible().unwrap_or(false);
                if is_visible {
                    let _ = launcher.hide();
                } else {
                    let _ = launcher.center();
                    let _ = launcher.show();
                    let _ = launcher.set_focus();
                }
            }
        }
    })?;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(main_window) = app.get_webview_window("main") {
                let _ = main_window.show();
                let _ = main_window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            hide_launcher,
            show_launcher,
            toggle_launcher,
            show_main_window,
            submit_message,
        ])
        .setup(|app| {
            // Setup system tray
            if let Err(e) = setup_tray(&app.handle()) {
                eprintln!("Failed to setup tray: {}", e);
            }
            
            // Setup global shortcut
            if let Err(e) = setup_global_shortcut(&app.handle()) {
                eprintln!("Failed to setup global shortcut: {}", e);
            }
            
            // Handle main window close - hide instead of quit
            if let Some(main_window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // Prevent close, hide instead
                        api.prevent_close();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                });
            }
            
            // Handle launcher losing focus - hide it
            if let Some(launcher) = app.get_webview_window("launcher") {
                let app_handle = app.handle().clone();
                launcher.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        if let Some(launcher_window) = app_handle.get_webview_window("launcher") {
                            let _ = launcher_window.hide();
                        }
                    }
                });
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
