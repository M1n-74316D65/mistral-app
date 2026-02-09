use tauri::{
    AppHandle, Manager, RunEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter,
};
#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;
#[cfg(target_os = "macos")]
#[allow(deprecated)]
use cocoa::appkit::{NSColor, NSWindow};
#[cfg(target_os = "macos")]
#[allow(deprecated)]
use cocoa::base::{id, nil, NO};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

// JavaScript to inject custom styles that hide UI elements overlapping with title bar
fn get_hide_titlebar_overlap_js() -> String {
    r#"
    (function() {
        const STYLE_ID = 'le-chat-custom-styles';
        
        function injectStyles() {
            // Avoid duplicate injection
            if (document.getElementById(STYLE_ID)) return;
            
            const style = document.createElement('style');
            style.id = STYLE_ID;
            style.textContent = `
                /* Hide the workspace menu button that overlaps title bar */
                div[data-sidebar="header"] button[aria-haspopup="menu"] {
                    display: none !important;
                }
                
                /* Hide the flex-1 wrapper containing the workspace button */
                div[data-sidebar="header"] .flex-1:has(button[aria-haspopup="menu"]) {
                    display: none !important;
                }
                
                /* Make the button container full width and push buttons to the right */
                div[data-sidebar="header"] > div.flex {
                    width: 100% !important;
                    justify-content: flex-end !important;
                }
                
                /* Add top padding to sidebar header to clear macOS traffic lights */
                div[data-sidebar="header"] {
                    padding-top: 2.5rem !important;
                }
            `;
            document.head.appendChild(style);
            
            // Fallback for browsers without :has() support
            document.querySelectorAll('div[data-sidebar="header"] button[aria-haspopup="menu"]').forEach(btn => {
                btn.style.display = 'none';
                // Also hide the flex-1 wrapper parent
                const wrapper = btn.closest('.flex-1');
                if (wrapper) {
                    wrapper.style.display = 'none';
                }
            });
            
            // Push buttons to the right (fallback)
            document.querySelectorAll('div[data-sidebar="header"] > div.flex').forEach(container => {
                container.style.width = '100%';
                container.style.justifyContent = 'flex-end';
            });
            
            console.log('[Le Chat] Custom styles injected');
        }
        
        // Retry until DOM is ready
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', injectStyles);
        } else {
            injectStyles();
        }
        
        // Re-inject on dynamic navigation (React SPA)
        new MutationObserver(() => injectStyles()).observe(
            document.documentElement, 
            { childList: true, subtree: true }
        );
    })();
    "#.to_string()
}

// JavaScript to inject custom titlebar for Windows and Linux (frameless window)
#[cfg(any(target_os = "windows", target_os = "linux"))]
fn get_custom_titlebar_js() -> String {
    r#"
    (function() {
        const TITLEBAR_ID = 'le-chat-custom-titlebar';
        
        function injectTitlebar() {
            if (document.getElementById(TITLEBAR_ID)) return;
            if (!document.body) {
                setTimeout(injectTitlebar, 50);
                return;
            }
            
            const titlebar = document.createElement('div');
            titlebar.id = TITLEBAR_ID;
            titlebar.setAttribute('data-tauri-drag-region', '');
            
            titlebar.innerHTML = `
                <style>
                    /* Mistral-themed titlebar with dark/light mode support */
                    #le-chat-custom-titlebar {
                        position: fixed;
                        top: 0;
                        left: 0;
                        right: 0;
                        height: 32px;
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        z-index: 999999;
                        -webkit-app-region: drag;
                        user-select: none;
                        transition: background-color 0.2s, border-color 0.2s;
                    }
                    
                    /* Dark mode (default for Mistral) */
                    #le-chat-custom-titlebar {
                        background: #18181b;
                        border-bottom: 1px solid #27272a;
                    }
                    #le-chat-custom-titlebar .title {
                        color: #a1a1aa;
                        font-size: 12px;
                        font-weight: 500;
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', sans-serif;
                        padding-left: 12px;
                        pointer-events: none;
                    }
                    #le-chat-custom-titlebar .controls {
                        display: flex;
                        height: 100%;
                        -webkit-app-region: no-drag;
                    }
                    #le-chat-custom-titlebar button {
                        width: 46px;
                        height: 100%;
                        border: none;
                        background: transparent;
                        color: #71717a;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        cursor: pointer;
                        transition: background-color 0.15s, color 0.15s;
                    }
                    #le-chat-custom-titlebar button:hover {
                        background: #27272a;
                        color: #fafafa;
                    }
                    #le-chat-custom-titlebar button:active {
                        background: #3f3f46;
                    }
                    #le-chat-custom-titlebar button.close:hover {
                        background: #dc2626;
                        color: #fff;
                    }
                    #le-chat-custom-titlebar button svg {
                        width: 10px;
                        height: 10px;
                    }
                    
                    /* Light mode support */
                    @media (prefers-color-scheme: light) {
                        #le-chat-custom-titlebar {
                            background: #fafafa;
                            border-bottom: 1px solid #e4e4e7;
                        }
                        #le-chat-custom-titlebar .title {
                            color: #52525b;
                        }
                        #le-chat-custom-titlebar button {
                            color: #71717a;
                        }
                        #le-chat-custom-titlebar button:hover {
                            background: #e4e4e7;
                            color: #18181b;
                        }
                        #le-chat-custom-titlebar button:active {
                            background: #d4d4d8;
                        }
                        #le-chat-custom-titlebar button.close:hover {
                            background: #dc2626;
                            color: #fff;
                        }
                    }
                    
                    /* Also detect Mistral's theme via html/body classes */
                    html.light #le-chat-custom-titlebar,
                    body.light #le-chat-custom-titlebar,
                    [data-theme="light"] #le-chat-custom-titlebar {
                        background: #fafafa;
                        border-bottom: 1px solid #e4e4e7;
                    }
                    html.light #le-chat-custom-titlebar .title,
                    body.light #le-chat-custom-titlebar .title,
                    [data-theme="light"] #le-chat-custom-titlebar .title {
                        color: #52525b;
                    }
                    html.light #le-chat-custom-titlebar button,
                    body.light #le-chat-custom-titlebar button,
                    [data-theme="light"] #le-chat-custom-titlebar button {
                        color: #71717a;
                    }
                    html.light #le-chat-custom-titlebar button:hover,
                    body.light #le-chat-custom-titlebar button:hover,
                    [data-theme="light"] #le-chat-custom-titlebar button:hover {
                        background: #e4e4e7;
                        color: #18181b;
                    }
                    html.light #le-chat-custom-titlebar button.close:hover,
                    body.light #le-chat-custom-titlebar button.close:hover,
                    [data-theme="light"] #le-chat-custom-titlebar button.close:hover {
                        background: #dc2626;
                        color: #fff;
                    }
                    
                    /* Push body content down to avoid overlap */
                    html, body {
                        padding-top: 32px !important;
                    }
                    /* Adjust sidebar header for Windows/Linux */
                    div[data-sidebar="header"] {
                        padding-top: 0.5rem !important;
                    }
                </style>
                <span class="title">Le Chat</span>
                <div class="controls">
                    <button id="lc-tb-minimize" title="Minimize">
                        <svg viewBox="0 0 10 1">
                            <rect fill="currentColor" width="10" height="1"/>
                        </svg>
                    </button>
                    <button id="lc-tb-maximize" title="Maximize">
                        <svg viewBox="0 0 10 10">
                            <rect fill="none" stroke="currentColor" stroke-width="1" width="9" height="9" x="0.5" y="0.5"/>
                        </svg>
                    </button>
                    <button id="lc-tb-close" class="close" title="Close">
                        <svg viewBox="0 0 10 10">
                            <path fill="currentColor" d="M1.41 0L0 1.41 3.59 5 0 8.59 1.41 10 5 6.41 8.59 10 10 8.59 6.41 5 10 1.41 8.59 0 5 3.59z"/>
                        </svg>
                    </button>
                </div>
            `;
            
            document.body.prepend(titlebar);
            
            // Wire up buttons using Tauri API
            const { getCurrentWindow } = window.__TAURI__.window;
            const appWindow = getCurrentWindow();
            
            document.getElementById('lc-tb-minimize').onclick = (e) => {
                e.preventDefault();
                appWindow.minimize();
            };
            document.getElementById('lc-tb-maximize').onclick = (e) => {
                e.preventDefault();
                appWindow.toggleMaximize();
            };
            document.getElementById('lc-tb-close').onclick = (e) => {
                e.preventDefault();
                appWindow.hide(); // Hide instead of close - app stays in tray
            };
            
            console.log('[Le Chat] Custom titlebar injected');
        }
        
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', injectTitlebar);
        } else {
            injectTitlebar();
        }
        
        // Re-inject on navigation (SPA)
        new MutationObserver(() => {
            if (!document.getElementById(TITLEBAR_ID)) {
                injectTitlebar();
            }
        }).observe(document.documentElement, { childList: true, subtree: true });
    })();
    "#.to_string()
}

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
            
// Make launcher window fully transparent on macOS for rounded corners
            #[cfg(target_os = "macos")]
            #[allow(deprecated, unexpected_cfgs)]
            {
                if let Some(launcher) = app.get_webview_window("launcher") {
                    if let Ok(ns_window) = launcher.ns_window() {
                        let ns_window = ns_window as id;
                        unsafe {
                            // Set window background to clear
                            let clear_color = NSColor::clearColor(nil);
                            ns_window.setBackgroundColor_(clear_color);
                            
                            // Disable WKWebView background drawing via private API
                            let content_view: id = msg_send![ns_window, contentView];
                            if !content_view.is_null() {
                                let subviews: id = msg_send![content_view, subviews];
                                let count: usize = msg_send![subviews, count];
                                for i in 0..count {
                                    let subview: id = msg_send![subviews, objectAtIndex:i];
                                    let _: () = msg_send![subview, _setDrawsBackground:NO];
                                }
                            }
                        }
                    }
                }
            }
            
            // Platform-specific window configuration and style injection
            if let Some(main_window) = app.get_webview_window("main") {
                #[cfg(target_os = "macos")]
                {
                    // macOS: Use overlay title bar style with hidden title
                    let _ = main_window.set_title_bar_style(TitleBarStyle::Overlay);
                    // Inject CSS to hide UI elements overlapping with traffic lights
                    let js = get_hide_titlebar_overlap_js();
                    let _ = main_window.eval(&js);
                }
                
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                {
                    // Windows/Linux: Remove native decorations and inject custom titlebar
                    let _ = main_window.set_decorations(false);
                    let js = get_custom_titlebar_js();
                    let _ = main_window.eval(&js);
                }
            }
            
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // Handle macOS dock icon click to reopen window
            if let RunEvent::Reopen { .. } = event {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });
}
