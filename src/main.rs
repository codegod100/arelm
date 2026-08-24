//! Desktop development entrypoint: `cargo run`.
//!
//! This is *not* what runs on Android - pixiewood links the `cdylib`
//! artifact (see src/android.rs) into a JNI library that GTK's Android
//! backend loads instead. Keeping a normal `fn main()` here just makes it
//! possible to iterate on the relm4 UI locally with a plain `cargo run`
//! before ever touching the Android toolchain.
fn main() {
    arelm_example::run();
}
