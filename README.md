# arelm

A proof-of-concept that packages a [relm4](https://relm4.org/) (reactive
Rust/GTK4) application for Android using
[gtk-android-builder](https://github.com/sp1ritCS/gtk-android-builder)
("pixiewood").

It's one small counter component, compiled twice from the same source:

- as a normal desktop binary, via `buck2 run //:arelm`;
- as a `cdylib` exporting a C `main()`, which is what pixiewood/GTK's
  Android backend loads as a JNI library on-device.

The whole crate graph - arelm's own code plus every transitive dependency
(relm4, the gtk4-rs bindings, the `-sys` crates underneath them, and
everything under *those*) - builds with **[buck2](https://buck2.build/)**.
There is no `cargo build`/`cargo run` anywhere in this repo's build graph,
for either the desktop or the Android target: `Cargo.toml`/`Cargo.lock`
exist only as the dependency-resolution source of truth that
[reindeer](https://github.com/facebookincubator/reindeer) reads to
generate real buck2 `rust_library` targets (see `third-party/BUCK`, and
"How the buck2 build is put together" below).

## Why this needs glue at all

pixiewood expects to build a **meson** project whose target is a C
`executable()` with `android_exe_type: 'application'` set - meson then
compiles it straight to a shared object exporting `main`, which GTK's own
Android glue (`gdk/android`, loaded into the app process via JNI) looks up
and calls after JNI/GLib setup.

relm4 apps aren't meson/C projects, they're **Rust** projects built with
buck2. So instead of using meson's C frontend at all, `meson.build` here
defines a `custom_target()` that shells out to `buck2 build
//:arelm-lib[cdylib]` for the right Android target platform, and installs
the resulting `.so` exactly where pixiewood expects it
(`get_option('libdir')`, which pixiewood's cross files repoint at
`lib/<abi>/` for the APK's `jniLibs`). Rust supplies the ABI contract
`main(int, char**, char**) -> int` (see `src/android.rs`) by hand instead of
via meson's `android_exe_type`, since that machinery never runs for us.

```
                     ┌─────────────────────┐
                     │      src/app.rs      │   one relm4 SimpleComponent
                     └──────────┬───────────┘
                    ┌────────────┴────────────┐
          src/main.rs                    src/android.rs
        (desktop `fn main`)      (cdylib `extern "C" fn main`,
                                   cfg(target_os = "android"))
                    │                            │
            `buck2 run //:arelm`        `buck2 build
                                     //:arelm-lib[cdylib]`,
                                     cross-compiled against
                                     `//platforms:android-
                                     {aarch64,x86_64}`, driven
                                     by meson.build's
                                     custom_target, in turn
                                     driven by `pixiewood build`
                                                  │
                                     packaged into an APK by
                                     pixiewood's generated
                                     Gradle project
```

## Layout

| Path | Purpose |
|---|---|
| `src/app.rs` | The actual relm4 UI (a counter). Plain GTK4, no libadwaita. |
| `src/main.rs` | Desktop entrypoint (`buck2 run //:arelm`). |
| `src/android.rs` | Android cdylib entrypoint: exports the C `main` pixiewood/GTK's Android glue calls. |
| `BUCK` | arelm's own `rust_library`/`rust_binary` targets (see "How the buck2 build is put together" below). |
| `platforms/BUCK` | `platform()`/`config_setting()` targets for `android-aarch64`/`android-x86_64`, keying every other `select()` in this repo. |
| `toolchains/BUCK` | The NDK-aware `rust`/`cxx` toolchains buck2 uses to actually cross-compile and link for Android. |
| `reindeer.toml`, `third-party/` | reindeer's config, fixups, vendored crate sources, and the generated `third-party/BUCK` - the entire third-party dependency graph as real buck2 targets. See below. |
| `meson.build` | Wraps `buck2 build //:arelm-lib[cdylib]` in a `custom_target` pixiewood can build & install like any other meson target. |
| `build-scripts/buck2-build-cdylib.sh` | The actual `buck2 build` invocation meson's custom_target runs. |
| `pixiewood.xml` | The pixiewood manifest (app id, icon, dependencies, build target, architectures). |
| `data/com.example.Arelm.metainfo.xml` | AppStream metadata (app id, version, icon branding colour). |
| `data/icons/*.xml` | Adaptive icon foreground/monochrome layers, hand-authored as Android VectorDrawables directly (see "Avoiding the Android Studio dependency" below) - pixiewood's `generate` step copies these in verbatim. |

## Desktop development

```sh
buck2 run //:arelm
```

This builds and runs `src/main.rs` against your system's GTK4 (via
`pkg-config`, same as before) - no Android SDK/NDK, meson, or pixiewood
involved. Iterate on `src/app.rs` here; the exact same component is what
gets cross-compiled for Android.

You can also drive it through the same `meson.build` pixiewood uses, which
is a good way to sanity-check that file in isolation:

```sh
meson setup builddir .
meson compile -C builddir
```

On a plain host build (no cross file), `dependency('gtk4')` in
`meson.build` resolves to your system GTK4 instead of pixiewood's
Android-cross-built subproject, so this works standalone and fast - meson
just shells out to `buck2 build //:arelm-lib[cdylib]` with no
`--target-platforms` override, which resolves to your host platform.

## Building the Android package

This needs, on top of what desktop development needs above:

- **gtk-android-builder** (pixiewood) itself - see its
  [README](https://github.com/sp1ritCS/gtk-android-builder#installation)
  for its own dependencies (Perl + XML/GLib bindings, Java 17, meson
  **≥ 1.9**, ninja, sassc, shaderc).
- **Android SDK** + an **NDK** under it (pixiewood auto-detects the newest
  one in `$ANDROID_HOME/ndk/`). `platforms;android-36` and
  `build-tools;36.0.0` specifically need to be installed via `sdkmanager` -
  the generated `app/build.gradle`'s `compileSdk` is 36, which won't
  necessarily match whatever platform you happened to install first.
  `toolchains/BUCK`'s `NDK_TOOLCHAIN_BIN` currently hardcodes an exact NDK
  version/path (see that file) - update it if your installed NDK differs.
- The Rust Android targets, for the *host* rustc buck2 invokes directly
  (there's no separate Android-specific rustc/cargo install - buck2 just
  passes `--target <triple>` and an NDK linker):
  ```sh
  rustup target add aarch64-linux-android x86_64-linux-android
  ```

Note this list does **not** include Android Studio, and does **not**
include `cargo` (only `rustc` itself, via the targets above - buck2 invokes
`rustc` directly). gtk-android-builder's own docs list Android Studio as a
prerequisite purely so its icon generator (`Svg2Avd`) can reuse jars from it
to convert SVG → Android vector drawable; this repo sidesteps that entirely
by hand-authoring the vector drawables directly (`data/icons/*.xml`) and
declaring them `type="avd"` in `pixiewood.xml`, which pixiewood copies in
verbatim with no conversion step. See the comment in
`data/icons/arelm-foreground.xml`.

Then, from this directory:

```sh
pixiewood prepare -s "$ANDROID_HOME" pixiewood.xml
pixiewood generate
pixiewood build
```

- `prepare` writes `subprojects/{glib,fontconfig,gtk}.wrap` (from
  `<dependencies>` in `pixiewood.xml`) and runs `meson setup` twice, once
  per architecture, with pixiewood's Android cross files.
- `generate` reads `pixiewood.xml` + the configured build dirs and produces
  the Gradle/Android Studio project under `.pixiewood/android/`.
- `build` runs `ninja` in each per-arch build dir (which is where
  `meson.build`'s `custom_target` invokes `buck2 build
  //:arelm-lib[cdylib] --target-platforms //platforms:android-{aarch64,x86_64}`),
  installs the runtime outputs into `.pixiewood/root`, and finally runs
  `./gradlew assembleDebug` to produce an APK under
  `.pixiewood/android/app/build/outputs/apk/`.

## How the buck2 build is put together

- **`third-party/`** is the entire transitive dependency graph (relm4, the
  gtk4-rs bindings, every `-sys` crate underneath them, and so on - about
  90 crates) as real buck2 `rust_library`/`buildscript_run` targets,
  generated by [reindeer](https://github.com/facebookincubator/reindeer)
  from `Cargo.toml`/`Cargo.lock`. `third-party/vendor/` (the vendored crate
  sources) and `third-party/BUCK` (the generated targets) are both
  committed, so a fresh clone can `buck2 build` immediately with no network
  access and no reindeer/cargo install needed. If you change
  `Cargo.toml`/`Cargo.lock`, regenerate both with:
  ```sh
  reindeer vendor
  reindeer buckify
  ```
  (reindeer's own config, `reindeer.toml`, sets `manifest_path =
  "Cargo.toml"` since this repo's manifest lives at the repo root rather
  than inside `third-party/` - reindeer's default assumes the latter, so
  don't drop that line.)
- **`third-party/fixups/*/fixups.toml`** patches individual crates' generated
  build rules - most importantly, the GTK `-sys` crates (`gtk4-sys`,
  `glib-sys`, etc.), whose `build.rs` shells out to `pkg-config` via the
  `system-deps` crate. `rustc_link_lib`/`rustc_link_search` turn their
  `cargo:rustc-link-lib=...` build-script output into real linker flags
  buck2 understands; a `cfg(target_os = "android", target_arch = "...")`
  platform fixup additionally sets `PKG_CONFIG_ALLOW_CROSS=1` and points
  `PKG_CONFIG_PATH` at pixiewood's own per-ABI, from-source GTK4 build tree
  (`.pixiewood/bin-<arch>/meson-uninstalled/`) instead of a system install
  - `pkg-config` refuses to cross-compile-probe at all without the former,
    and there is no system-installed GTK4 for Android in the first place.
- **`third-party/PACKAGE`** tells reindeer's generated targets which
  `reindeer.toml` platform (`linux-x86_64`, `android-aarch64`,
  `android-x86_64`) is active for the current buck2 target configuration,
  keyed off the same `//platforms:is-android-*` `config_setting`s
  `toolchains/BUCK` and `BUCK` key off - this is what makes the
  `cfg(...)`-gated fixups above actually apply only when cross-compiling.
- **`toolchains/BUCK`** overrides both the `rust` and `cxx` system
  toolchains to be NDK-aware: `rustc_target_triple` picks the right Android
  triple, and (since buck2's rust rules always link through the *cxx*
  toolchain's linker, not a `-Clinker=` rustc flag) the `cxx` toolchain's
  `compiler`/`cxx_compiler`/`linker`/`archiver` point at the NDK's own
  per-ABI clang/llvm-ar wrapper instead of the host's system compiler -
  both `select()`-ed on `//platforms:is-android-{aarch64,x86_64}`.
- **`BUCK`** (this directory) defines arelm's own `rust_library`
  (`:arelm-lib`, exposing a `[cdylib]` subtarget - see
  `@prelude//rust/rust_library.bzl`'s automatic per-crate-type subtargets)
  and `rust_binary` (`:arelm`), both depending on `//third-party:relm4`.
  Since none of the `-sys` crates' build scripts emit
  `cargo:rustc-link-search` (only `-link-lib`), `:arelm-lib`/`:arelm` add
  the missing `-L` search-path flags explicitly, `select()`-ed the same way
  between the host's GTK4 install and pixiewood's per-ABI build tree.

## What's been verified

**buck2 build, both desktop and Android:**

- `buck2 build //:arelm-lib //:arelm-lib[cdylib] //:arelm` succeeds against
  the host's system GTK4 (via `pkg-config`), and the resulting `:arelm`
  binary links cleanly (`ldd` resolves every GTK4 shared library, no
  "not found" entries).
- `buck2 build --target-platforms //platforms:android-aarch64
  //:arelm-lib[cdylib]` (and the `android-x86_64` equivalent) both succeed,
  cross-compiling the entire relm4/gtk4-rs stack - including every GTK
  `-sys` crate's `pkg-config`-based build script - against pixiewood's
  from-source Android GTK4 build. The resulting `.so` is a real `ELF 64-bit
  ... ARM aarch64 ... for Android 31, built by NDK r27c`, linked against
  `libgtk-4.so`/`libglib-2.0.so`/etc. with no host libraries leaking in
  (confirmed via `file`/`readelf -d`).
- `meson.build`'s `custom_target` (both the plain host path and a
  simulated `--rust-target aarch64-linux-android --profile release`
  invocation of `build-scripts/buck2-build-cdylib.sh` directly) produces
  the same working `.so` in both cases.

**Not yet re-verified since the buck2 migration:** a full physical-device
run - `pixiewood build` driving buck2 end-to-end, through to an installed,
launched APK - as was previously done with the old cargo-based build (see
git history for that run's details, against a Pixel 6a). The buck2-built
Android `.so` above hasn't yet been carried through `pixiewood build`'s
Gradle packaging step and installed on-device; that's the next thing to
confirm.

## License

GPL-3.0-or-later, matching gtk-android-builder.
