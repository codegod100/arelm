pub mod app;

/// Must match the `<component><id>` in the AppStream metainfo and the
/// package id derived from it in `pixiewood.xml` (pixiewood turns the
/// metainfo id into both the Android application id and the Java package,
/// see README.md).
pub const APP_ID: &str = "com.example.Arelm";

/// Shared entrypoint used by both the desktop binary and the Android cdylib.
/// This is the call that, transitively, reaches `g_application_run()` -
/// `RelmApp::run` builds a `gtk::Application` (a `gio::Application`
/// subclass) and calls its `run()`.
pub fn run() {
    let app = relm4::RelmApp::new(APP_ID);
    app.run::<app::AppModel>(0);
}

// Only compiled when cross-compiling the cdylib for Android; on desktop
// targets the crate only exposes the normal Rust `run()` used by src/main.rs.
#[cfg(target_os = "android")]
mod android;
