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
| `data/icons/*.xml` | Adaptive icon foreground/monochrome layers, hand-authored as Android VectorDrawables directly (see "Avoiding the Android Studio dependency" below) - pixiewood's `generate` step copies these in verbatim. |

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
  one in `$ANDROID_HOME/ndk/`). `platforms;android-36` and
  `build-tools;36.0.0` specifically need to be installed via `sdkmanager` -
  the generated `app/build.gradle`'s `compileSdk` is 36, which won't
  necessarily match whatever platform you happened to install first.
- The Rust Android targets:
  ```sh
  rustup target add aarch64-linux-android x86_64-linux-android
  ```
  If you also have Homebrew's own `cargo`/`rustc` formula installed and its
  `bin` dir earlier in `$PATH` than rustup's, cross builds fail with `error[E0463]:
  can't find crate for core` even with the targets correctly installed -
  brew's cargo doesn't know about rustup's targets. Make sure
  `$HOME/.cargo/bin` (or wherever your rustup shims live) comes first in
  `$PATH`.

Note this list does **not** include Android Studio. gtk-android-builder's
own docs list it as a prerequisite purely so its icon generator (`Svg2Avd`)
can reuse jars from it to convert SVG → Android vector drawable; this repo
sidesteps that entirely by hand-authoring the vector drawables directly
(`data/icons/*.xml`) and declaring them `type="avd"` in `pixiewood.xml`,
which pixiewood copies in verbatim with no conversion step. See the comment
in `data/icons/arelm-foreground.xml`.

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

## What's actually been verified here

The full pipeline has been run for real, end-to-end, and the resulting APK
installed and exercised on a physical device:

- `cargo build` / `cargo run` (`src/main.rs`, `src/app.rs`) compile clean
  against a real GTK4 4.20 and pull in relm4 0.11 from crates.io.
- `meson.build` - the *exact* file pixiewood drives - was run standalone
  (`meson setup builddir . && meson compile -C builddir`) against the host's
  GTK4 as a quick sanity check before involving pixiewood at all.
- `pixiewood.xml` validates against the upstream `pixiewood.xsd` schema, and
  `pixiewood prepare -s "$ANDROID_HOME" pixiewood.xml` ran for real for both
  `aarch64` and `x86_64`: it wrote the `subprojects/*.wrap` files and ran
  `meson setup` with pixiewood's Android cross files, cross-compiling
  glib/fontconfig/gtk4's full from-source dependency chain against the NDK.
- `pixiewood generate` produced a working Gradle/Android Studio project
  under `.pixiewood/android/`, with the expected wiring confirmed by
  reading the generated files directly: `AndroidManifest.xml`'s
  `gtk.android.lib_name` meta-data pointing at `arelm`, `app/build.gradle`'s
  `applicationId "com.example.arelm"`, and the hand-authored icon XML files
  copied in verbatim as `res/drawable/ic_launcher_{foreground,monochrome}.xml`.
- `pixiewood build` ran `ninja` per architecture (compiling the entire GTK4
  stack - glib, pango, cairo, harfbuzz, fontconfig, gdk-pixbuf, gtk4 - from
  source for Android, plus the `custom_target` that cross-compiles
  `libarelm.so` via cargo), installed everything into `.pixiewood/root`, and
  ran `./gradlew assembleDebug` to a clean `BUILD SUCCESSFUL`.
- The resulting `app-arm64-v8a-debug.apk` was installed with `adb install`
  and launched with `adb shell am start` on a physical **Pixel 6a (Android
  16, arm64-v8a)**. `dumpsys` confirmed it as the foreground activity, and
  on-device screenshots confirm the relm4 counter UI renders correctly and
  responds to real touch input: tapping "Increment" three times took the
  displayed count from 0 to 3.

A few real issues were found and fixed along the way (see git history for
details): a `pixiewood.xml` manifest bug where `build://{arch}/...` had been
copied from gtk-android-builder's README as if `{arch}` were literal
templating syntax (it's just doc-placeholder notation - pixiewood requires a
real, concrete architecture name there); a missing `platforms;android-36`
SDK install (the generated `build.gradle`'s `compileSdk` doesn't
automatically match whatever platform you install first); and a PATH-
ordering issue where Homebrew's own `cargo` shadowed rustup's, breaking
Android cross-compilation despite the targets being correctly installed.

## License

GPL-3.0-or-later, matching gtk-android-builder.
