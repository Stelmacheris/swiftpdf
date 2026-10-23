//! Tauri shell for SwiftPDF. Keeps only window and IPC wiring; all document logic
//! lives in the `swiftpdf-core` crate.

use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

/// Benchmark hook for `bench/cold-start.ps1`. Off by default: only when this variable names a
/// file are startup marks appended to it, as `<mark> <microseconds since the Unix epoch>`.
const STARTUP_LOG_ENV: &str = "SWIFTPDF_STARTUP_LOG";

/// Startup mark written when the UI has painted its first frame (content visible).
const CONTENT_READY_MARK: &str = "content_ready";

/// Called by the UI right after its first paint. The window itself is already visible: it is
/// created shown with the theme background (T-111), so the user sees a dark frame at once instead
/// of waiting about 500 ms for WebView2. This call only marks the moment content is on screen.
#[tauri::command]
fn window_ready() {
    log_startup_mark(CONTENT_READY_MARK);
}

/// Appends a timestamped startup mark to the file named by `SWIFTPDF_STARTUP_LOG`, if set.
fn log_startup_mark(mark: &str) {
    let Some(path) = std::env::var_os(STARTUP_LOG_ENV) else {
        return;
    };
    let Ok(since_epoch) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return;
    };
    let line = format!("{mark} {}\n", since_epoch.as_micros());
    // Timing is best effort; a failed write must never affect the app.
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(line.as_bytes()));
}

/// Builds and runs the Tauri application.
pub fn run() {
    log_startup_mark("main");
    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![window_ready])
        .setup(|_app| {
            // Tauri runs this hook after it has built the config windows and their WebView2.
            log_startup_mark("webview_built");
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

    /// Label of the main window in tauri.conf.json.
    const MAIN_WINDOW: &str = "main";

    /// Dark theme `--bg-app` token (docs/design/theme.md); dark is the default theme.
    const DARK_BG_APP: &str = "#1b1d21";

    fn config() -> Value {
        serde_json::from_str(include_str!("../tauri.conf.json"))
            .expect("tauri.conf.json is valid JSON")
    }

    fn main_window(config: &Value) -> &Value {
        &config["app"]["windows"][0]
    }

    #[test]
    fn main_window_is_titled_swiftpdf() {
        let config = config();
        let window = main_window(&config);
        assert_eq!(window["label"], MAIN_WINDOW);
        assert_eq!(window["title"], swiftpdf_core::PRODUCT_NAME);
    }

    /// T-111: the frame is shown at creation, painted in the theme background so it never
    /// flashes white while WebView2 starts. Content readiness is reported by `window_ready`.
    #[test]
    fn main_window_starts_visible_with_the_dark_theme_background() {
        let config = config();
        let window = main_window(&config);
        assert_eq!(window["visible"], true);
        assert_eq!(window["theme"], "Dark");
        assert_eq!(
            window["backgroundColor"]
                .as_str()
                .map(str::to_ascii_lowercase),
            Some(DARK_BG_APP.to_string())
        );
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

    /// T-007: the fetch script pins one non-V8, non-XFA archive per architecture by SHA-256.
    #[test]
    fn fetch_pdfium_pins_non_v8_archives_by_sha256() {
        let script = include_str!("../../scripts/fetch-pdfium.ps1");
        assert!(
            !script.contains("pdfium-v8-"),
            "only the non-V8 PDFium build may be fetched"
        );
        assert!(script.contains("pdf_enable_v8") && script.contains("pdf_enable_xfa"));
        for asset in ["pdfium-win-x64.tgz", "pdfium-win-arm64.tgz"] {
            let line = script
                .lines()
                .find(|line| line.contains(&format!("File = '{asset}'")))
                .unwrap_or_else(|| panic!("{asset} is not pinned"));
            let sha = line
                .split("Sha256 = '")
                .nth(1)
                .and_then(|rest| rest.split('\'').next())
                .unwrap_or_default();
            assert_eq!(sha.len(), 64, "{asset} needs a full SHA-256");
            assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
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
