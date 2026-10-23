//! Tauri shell for SwiftPDF. Keeps only window and IPC wiring; all document logic
//! lives in the `swiftpdf-core` crate.

use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime};

/// Label of the main window in tauri.conf.json.
const MAIN_WINDOW: &str = "main";

/// The window is created hidden and shown when the UI reports its first paint.
/// If the UI never reports (for example a script error), show it anyway after this delay.
const SHOW_FALLBACK_DELAY: Duration = Duration::from_secs(3);

/// Shows the main window. Called by the UI right after its first paint, so the user
/// never sees an empty white WebView.
#[tauri::command]
fn window_ready<R: Runtime>(app: AppHandle<R>) {
    show_main_window(&app);
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };
    // Showing an already visible window is a no-op, so the fallback and the UI call can race safely.
    if let Err(error) = window.show() {
        eprintln!("swiftpdf: could not show the main window: {error}");
    }
}

fn spawn_show_fallback<R: Runtime>(app: AppHandle<R>) {
    std::thread::spawn(move || {
        std::thread::sleep(SHOW_FALLBACK_DELAY);
        show_main_window(&app);
    });
}

/// Builds and runs the Tauri application.
pub fn run() {
    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![window_ready])
        .setup(|app| {
            spawn_show_fallback(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!(
            "swiftpdf: {} failed to start: {error}",
            swiftpdf_core::PRODUCT_NAME
        );
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    fn config() -> Value {
        serde_json::from_str(include_str!("../tauri.conf.json"))
            .expect("tauri.conf.json is valid JSON")
    }

    fn main_window(config: &Value) -> &Value {
        &config["app"]["windows"][0]
    }

    #[test]
    fn main_window_is_titled_swiftpdf_and_starts_hidden() {
        let config = config();
        let window = main_window(&config);
        assert_eq!(window["label"], super::MAIN_WINDOW);
        assert_eq!(window["title"], swiftpdf_core::PRODUCT_NAME);
        assert_eq!(window["visible"], false);
    }

    #[test]
    fn csp_has_no_remote_origins() {
        let config = config();
        let csp = config["app"]["security"]["csp"]
            .as_str()
            .expect("a CSP string is set");
        assert!(csp.contains("default-src 'self'"));
        assert!(
            !csp.contains("https:"),
            "CSP must not allow https origins: {csp}"
        );
        assert!(!csp.contains("*"), "CSP must not use wildcards: {csp}");
        assert!(
            !csp.contains("unsafe-eval"),
            "CSP must not allow eval: {csp}"
        );
        // Only Tauri's local pseudo-hosts may appear as http origins.
        for token in csp.split_whitespace() {
            let origin = token.trim_end_matches(';');
            if let Some(host) = origin.strip_prefix("http://") {
                assert!(
                    host.ends_with(".localhost"),
                    "remote origin in CSP: {origin}"
                );
            }
        }
    }

    #[test]
    fn capabilities_grant_no_plugins() {
        let capability: Value =
            serde_json::from_str(include_str!("../capabilities/default.json")).expect("valid JSON");
        let permissions = capability["permissions"]
            .as_array()
            .expect("permissions array");
        for permission in permissions {
            let name = permission.as_str().expect("permission is a string");
            assert!(
                !name.contains(':') || name.starts_with("core:"),
                "plugin permission not allowed in the scaffold: {name}"
            );
        }
    }
}
