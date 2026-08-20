# arelm's own crate: one `rust_library` (mirrors Cargo.toml's single `[lib]`
# with `crate-type = ["cdylib", "rlib"]`) plus one `rust_binary` for the
# desktop entrypoint. Every crate-type buck2's rust rules can produce from a
# single `rust_library` is exposed as a named subtarget automatically (see
# @prelude//rust/rust_library.bzl's `_SUB_TARGET_BUILD_LANG_STYLE`) - no
# separate target or `crate_type` attribute needed:
#
#  - `buck2 build //:arelm-lib`          - plain rlib (default output).
#  - `buck2 build //:arelm-lib[cdylib]`  - the cdylib pixiewood/GTK's
#                                          Android glue loads via JNI, cross-
#                                          compiled per Android ABI by
#                                          selecting `platforms//:android-
#                                          {aarch64,x86_64}` (see meson.build,
#                                          which invokes this instead of
#                                          `cargo build`; src/android.rs is
#                                          only pulled in by src/lib.rs's
#                                          `#[cfg(target_os = "android")]`
#                                          when actually building for one of
#                                          those platforms).
#  - `buck2 run //:arelm`                - the desktop binary, replacing
#                                          `cargo run`.
#
# Both link against the same third-party/BUCK (reindeer-buckified) crate
# graph reindeer.toml describes - one dependency edge, `:relm4`, pulls in
# the entire gtk4-rs stack transitively. `:relm4` (not the versioned
# `:relm4-0.11`) is reindeer's own generated public alias for the crate -
# the versioned target itself defaults to private (visibility = []).
#
# The gtk4-sys/glib-sys/etc fixups (third-party/fixups/*/fixups.toml) turn on
# `rustc_link_lib` so each -sys crate's `cargo:rustc-link-lib=gtk-4` etc.
# build.rs output becomes real `-l` flags on *that* crate's own rustc
# invocation - rustc embeds those as native-library requirements in the
# rlib's metadata, so they propagate automatically to whatever finally links
# against it (no `-l` flags needed here). But system-deps' build.rs never
# emits `cargo:rustc-link-search` at all (on this machine GTK4 only exists
# under Homebrew's non-standard Cellar paths, not a linker default dir) -
# `-l` embedding doesn't carry `-L` along with it, so the *actual link*
# steps below (this cdylib subtarget, and :arelm's binary link) need those
# search paths passed explicitly.
HOMEBREW_GTK4_LIB_DIRS = [
    "-L/home/linuxbrew/.linuxbrew/lib",
    "-L/home/linuxbrew/.linuxbrew/Cellar/gtk4/4.22.4/lib",
    "-L/home/linuxbrew/.linuxbrew/Cellar/pango/1.58.2/lib",
    "-L/home/linuxbrew/.linuxbrew/Cellar/harfbuzz/14.3.1/lib",
    "-L/home/linuxbrew/.linuxbrew/Cellar/cairo/1.18.4/lib",
    "-L/home/linuxbrew/.linuxbrew/Cellar/graphene/1.10.8/lib",
    "-L/home/linuxbrew/.linuxbrew/Cellar/glib/2.88.3/lib",
]

# Same story as HOMEBREW_GTK4_LIB_DIRS above, but for pixiewood's own
# from-source, per-ABI GTK4 build tree instead of the system one - each
# subproject's build dir holds its .so right next to the *-uninstalled.pc
# file that names it (see .pixiewood/bin-<arch>/meson-uninstalled/*.pc,
# `Libs: -L<dir> -l<name>`). PKG_CONFIG_PATH pointed at that same
# meson-uninstalled dir (see third-party/fixups/*/fixups.toml's
# `buildscript.run.env` platform_fixup) gets the `-l` flags onto the -sys
# crates themselves via `rustc_link_lib`; these `-L` dirs are for the actual
# link step here, same reasoning as the desktop dirs above. BUCK files (as
# opposed to .bzl) can't define `def`s, so this is spelled out per-arch
# instead of via a shared helper.
PIXIEWOOD_AARCH64_LIB_DIRS = [
    "-L.pixiewood/bin-aarch64/subprojects/gtk/gtk",
    "-L.pixiewood/bin-aarch64/subprojects/glib/glib",
    "-L.pixiewood/bin-aarch64/subprojects/glib/gobject",
    "-L.pixiewood/bin-aarch64/subprojects/glib/gio",
    "-L.pixiewood/bin-aarch64/subprojects/pango/pango",
    "-L.pixiewood/bin-aarch64/subprojects/cairo/src",
    "-L.pixiewood/bin-aarch64/subprojects/cairo/util/cairo-gobject",
    "-L.pixiewood/bin-aarch64/subprojects/gdk-pixbuf/gdk-pixbuf",
    "-L.pixiewood/bin-aarch64/subprojects/graphene/src",
    "-L.pixiewood/bin-aarch64/subprojects/harfbuzz/src",
    "-L.pixiewood/bin-aarch64/subprojects/proxy-libintl-0.5",
]

PIXIEWOOD_X86_64_LIB_DIRS = [
    "-L.pixiewood/bin-x86_64/subprojects/gtk/gtk",
    "-L.pixiewood/bin-x86_64/subprojects/glib/glib",
    "-L.pixiewood/bin-x86_64/subprojects/glib/gobject",
    "-L.pixiewood/bin-x86_64/subprojects/glib/gio",
    "-L.pixiewood/bin-x86_64/subprojects/pango/pango",
    "-L.pixiewood/bin-x86_64/subprojects/cairo/src",
    "-L.pixiewood/bin-x86_64/subprojects/cairo/util/cairo-gobject",
    "-L.pixiewood/bin-x86_64/subprojects/gdk-pixbuf/gdk-pixbuf",
    "-L.pixiewood/bin-x86_64/subprojects/graphene/src",
    "-L.pixiewood/bin-x86_64/subprojects/harfbuzz/src",
    "-L.pixiewood/bin-x86_64/subprojects/proxy-libintl-0.5",
]

# No `_rust_toolchain` override needed here: `toolchains//:rust` (see
# toolchains/BUCK) is itself select()-ed to be NDK-aware for Android target
# platforms - every rust_library/rust_binary's default toolchain dep already
# picks up the right rustc_target_triple/linker automatically, which matters
# because the reindeer-generated third-party crates below (:relm4 and
# everything it pulls in) have no way to override their toolchain per-target.
GTK4_LIB_DIRS = select({
    "//platforms:is-android-aarch64": PIXIEWOOD_AARCH64_LIB_DIRS,
    "//platforms:is-android-x86_64": PIXIEWOOD_X86_64_LIB_DIRS,
    "DEFAULT": HOMEBREW_GTK4_LIB_DIRS,
})

# buck2 has no built-in cargo-style dev/release profile concept - this is the
# standard way a caller (build-scripts/buck2-build-cdylib.sh, invoked from
# meson.build, passes `--config arelm.release=true` when meson's own
# buildtype is "release") threads that choice into a BUCK file. Mirrors
# cargo's `--release` (`-Copt-level=3`, cargo's own default release profile)
# closely enough for this PoC without trying to replicate cargo's full
# release profile defaults (LTO, codegen-units, etc. are left at rustc's
# stock settings).
RUSTC_RELEASE_FLAGS = ["-Copt-level=3"] if read_root_config("arelm", "release", "false") == "true" else []

rust_library(
    name = "arelm-lib",
    srcs = ["src/android.rs", "src/app.rs", "src/lib.rs"],
    crate = "arelm",
    crate_root = "src/lib.rs",
    edition = "2021",
    rustc_flags = GTK4_LIB_DIRS + RUSTC_RELEASE_FLAGS,
    visibility = ["PUBLIC"],
    deps = ["//third-party:relm4"],
)

rust_binary(
    name = "arelm",
    srcs = ["src/main.rs"],
    crate = "arelm",
    crate_root = "src/main.rs",
    edition = "2021",
    rustc_flags = HOMEBREW_GTK4_LIB_DIRS,
    visibility = ["PUBLIC"],
    # Cargo auto-links a package's own `[lib]` into its `[[bin]]` by crate
    # name (both are named "arelm" here; src/main.rs calls `arelm::run()`
    # as if it were an external crate) - buck2 has no equivalent implicit
    # wiring, so depend on `:arelm-lib` explicitly instead of duplicating
    # app.rs into this target the way the first draft did.
    deps = [":arelm-lib"],
)
