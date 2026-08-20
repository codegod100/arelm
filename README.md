# arelm

A proof-of-concept that packages a [relm4](https://relm4.org/) (reactive
Rust/GTK4) application for Android using
[gtk-android-builder](https://github.com/sp1ritCS/gtk-android-builder)
("pixiewood").

It's one small counter component, compiled twice from the same source:

- as a normal desktop binary, for `cargo run`;
- as a `cdylib` exporting a C `main()`, which is what pixiewood/GTK's
  Android backend loads as a JNI library on-device.

## Why this needs glue at all

pixiewood expects to build a **meson** project whose target is a C
`executable()` with `android_exe_type: 'application'` set - meson then
compiles it straight to a shared object exporting `main`, which GTK's own
Android glue (`gdk/android`, loaded into the app process via JNI) looks up
and calls after JNI/GLib setup.

relm4 apps aren't meson/C projects, they're **cargo** projects. So instead
of using meson's C frontend at all, `meson.build` here defines a
`custom_target()` that shells out to `cargo build` for the right Android
target triple, and installs the resulting `.so` exactly where pixiewood
expects it (`get_option('libdir')`, which pixiewood's cross files repoint
at `lib/<abi>/` for the APK's `jniLibs`). Rust supplies the ABI contract
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
              `cargo run`               cargo built for
                                    aarch64/x86_64-linux-android
                                       by meson.build's
                                       custom_target, driven
                                       by `pixiewood build`
                                                  │
                                     packaged into an APK by
                                     pixiewood's generated
                                     Gradle project
```

## Layout

| Path | Purpose |
|---|---|
| `src/app.rs` | The actual relm4 UI (a counter). Plain GTK4, no libadwaita. |
| `src/main.rs` | Desktop entrypoint (`cargo run`). |
| `src/android.rs` | Android cdylib entrypoint: exports the C `main` pixiewood/GTK's Android glue calls. |
| `meson.build` | Wraps `cargo` in a `custom_target` pixiewood can build & install like any other meson target. |
| `build-scripts/cargo-build-cdylib.sh` | The actual `cargo build` invocation meson's custom_target runs. |
| `pixiewood.xml` | The pixiewood manifest (app id, icon, dependencies, build target, architectures). |
| `data/com.example.Arelm.metainfo.xml` | AppStream metadata (app id, version, icon branding colour). |
| `data/icons/*.svg` | Adaptive icon foreground/monochrome layers pixiewood's `generate` step converts to Android vector drawables. |

## Desktop development

```sh
cargo run
```

This builds and runs `src/main.rs` against your system's GTK4 - no
Android SDK/NDK, meson, or pixiewood involved. Iterate on `src/app.rs`
here; the exact same component is what gets cross-compiled for Android.

You can also drive it through the same `meson.build` pixiewood uses, which
is a good way to sanity-check that file in isolation:

```sh
meson setup builddir .
meson compile -C builddir
```

On a plain host build (no cross file), `dependency('gtk4')` in
`meson.build` resolves to your system GTK4 instead of pixiewood's
Android-cross-built subproject, so this works standalone and fast.

## Building the Android package

This needs, on top of what `cargo run` needs above:

- **gtk-android-builder** (pixiewood) itself - see its
  [README](https://github.com/sp1ritCS/gtk-android-builder#installation)
  for its own dependencies (Perl + XML/GLib bindings, Java 17, meson
  **≥ 1.9**, ninja, sassc, shaderc).
- **Android SDK** + an **NDK** under it (pixiewood auto-detects the newest
  one in `$ANDROID_HOME/ndk/`).
- **Android Studio** installed, purely because pixiewood's icon generator
  (`Svg2Avd`) reuses jar files from it to convert SVG → Android vector
  drawable (`--android-studio-dir`, default `/opt/android-studio/`).
- The Rust Android targets:
  ```sh
  rustup target add aarch64-linux-android x86_64-linux-android
  ```

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
  `meson.build`'s `custom_target` invokes `cargo build --target
  {aarch64,x86_64}-linux-android`), installs the runtime outputs into
  `.pixiewood/root`, and finally runs `./gradlew assembleDebug` to produce
  an APK under `.pixiewood/android/app/build/outputs/apk/`.

### How `meson.build` wires cargo to pixiewood's cross-built GTK

The tricky part isn't invoking `cargo build --target
aarch64-linux-android` - it's that the `gtk4`/`glib`/etc. Rust crates need
to *link against* pixiewood's own from-source, Android-patched GTK build,
which doesn't exist as an installed system package on Android and, at the
point ninja would run our `custom_target`, hasn't even been `meson
install`-ed yet (pixiewood only does that after the whole `ninja` build
finishes). `meson.build` handles this by, only when cross-compiling:

1. Depending on the actual `libgtk` build target GTK's own meson.build
   exposes (via `subproject('gtk').get_variable('libgtk')`), so ninja
   builds GTK - and transitively glib/pango/cairo/etc. - before us.
2. Pointing `PKG_CONFIG_PATH` at `<builddir>/meson-uninstalled/`, where
   every subproject's `pkg.generate()` call writes a `*-uninstalled.pc`
   with `-L`/`-I` flags into the *build tree* as soon as it's configured -
   exactly what the `gtk4-sys`/`glib-sys`/etc. crates' `system-deps`-based
   build scripts need.
3. Reusing the exact NDK clang meson itself resolved for this cross build
   (`meson.get_compiler('c')`) as cargo's linker, via
   `CARGO_TARGET_<TRIPLE>_LINKER`, instead of re-deriving the NDK's
   directory layout independently.

## What's actually been verified here, and what hasn't

This was put together and checked in a sandbox with no Android SDK, NDK, or
Android Studio available, so the real `pixiewood prepare/generate/build`
pipeline above has **not** been run end-to-end. What has been concretely
verified instead:

- `cargo build` / `cargo run` (`src/main.rs`, `src/app.rs`) compile clean
  against a real GTK4 4.20 and pull in relm4 0.11 from crates.io.
- `meson.build` - the *exact* file pixiewood would drive - was run for
  real: `meson setup builddir . && meson compile -C builddir` successfully
  invoked `build-scripts/cargo-build-cdylib.sh`, which built `libarelm.so`
  via cargo and meson placed it as `custom_target`'s declared output; `meson
  install --tags runtime` then installed it correctly (meson's install-tag
  auto-guesser recognises a `.so` under `libdir` as tag `runtime`, so no
  file was silently skipped by pixiewood's `--tags runtime` install).
- `pixiewood.xml` validates against the upstream `pixiewood.xsd` schema.
- The `libgtk` subproject-variable dependency and
  `meson.override_dependency('gtk4', libgtk_dep)` wiring referenced above
  were confirmed by reading GTK's actual `meson.build`
  (`gitlab.gnome.org/GNOME/gtk`), not assumed.

What's *not* verified, because it needs tooling this sandbox doesn't have:

- An actual cross-compiled build against pixiewood's Android-patched GTK
  subproject (this would also mean building GTK, glib, cairo, pango,
  harfbuzz, fontconfig, gdk-pixbuf from source for Android - a very long
  build even with the toolchain available).
- Running `pixiewood generate`/`build` for real, and therefore whether the
  generated Gradle project actually assembles and installs on-device.
- The `Svg2Avd`-based icon generation, which needs real Android Studio
  jars.

## License

GPL-3.0-or-later, matching gtk-android-builder.
