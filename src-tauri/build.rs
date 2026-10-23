fn main() {
    // Declaring the app commands generates an `allow-<command>` permission for each,
    // so capabilities/default.json can grant exactly the commands the UI uses.
    let manifest = tauri_build::AppManifest::new().commands(&["window_ready"]);
    let attributes = tauri_build::Attributes::new().app_manifest(manifest);
    tauri_build::try_build(attributes).expect("failed to run tauri-build");
}
