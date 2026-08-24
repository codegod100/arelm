//! Arelm's counter application.
//!
//! This is an example consumer of the public `arelm` support library, not
//! part of that library's API.

#[path = "app.rs"]
pub mod app;

pub const APP_ID: &str = "com.example.Arelm";

pub fn run() {
    let app = arelm::relm4::RelmApp::new(APP_ID);
    app.run::<app::AppModel>(0);
}

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod android;
