//! Android entrypoint.
//!
//! gtk-android-builder's README states the precondition for any app it
//! packages:
//!
//! > Have an exposed `main(int, char**, char**)` entrypoint (reduced
//! > parameters are allowed) that ends up calling `g_application_run`
//! > before it returns
//!
//! That precondition is written for C apps compiled directly by meson
//! (`executable(..., android_exe_type: 'application')`), where meson's
//! Android support turns the "executable" into a shared object exporting
//! `main`, which GTK's own Android glue (`gdk/android`, loaded by the Java
//! `GtkApplication`/`GtkActivity` via JNI) looks up with `dlsym` and calls
//! on a worker thread after JNI/GLib setup.
//!
//! We are not compiling with meson's C frontend at all (see meson.build,
//! which wraps `cargo` in a `custom_target` instead), so meson's
//! `android_exe_type` machinery doesn't apply to us - but the *ABI contract*
//! it exists to satisfy is identical either way: the resulting `.so` must
//! export a C symbol literally named `main` with this signature. So we
//! provide it by hand here, and forward straight into the same relm4
//! `run()` used by the desktop binary.
use std::os::raw::{c_char, c_int};

#[no_mangle]
pub extern "C" fn main(_argc: c_int, _argv: *mut *mut c_char, _envp: *mut *mut c_char) -> c_int {
    // relm4's `RelmApp::run` builds a `gtk::Application` (a `gio::Application`
    // subclass) and calls its `.run()`, which is what actually reaches
    // `g_application_run()` on the C side - satisfying pixiewood's
    // precondition above.
    crate::run();
    0
}
