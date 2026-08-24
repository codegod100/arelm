//! Reusable Relm4/GTK Android support cell.
//!
//! Applications own their UI, application ID, Android `main` symbol and
//! `cdylib`. Re-exporting Relm4 gives consumers one version-coherent API
//! backed by this cell's Android-vetted dependency graph.

pub use relm4;
