project(
  'arelm', 'c',
  version: '0.1.0',
  meson_version: '>=1.9',
)

# gtk-android-builder (pixiewood) writes subprojects/{glib,fontconfig,gtk}.wrap
# for us - generated from the <dependencies> listed in pixiewood.xml - before
# it ever runs `meson setup`. Each wrap declares `[provide] dependency_names
# = ...`, so the plain dependency() calls below transparently resolve to
# those subprojects *only when needed*, i.e. when cross-compiling for
# Android, where there is no system GTK4 for pkg-config to find in the first
# place. On a normal desktop host (no pixiewood, no cross file, no
# subprojects/ directory) the very same calls just resolve to the
# system-installed GTK4 instead, which is what makes
#   meson setup builddir . && meson compile -C builddir
# on this exact file a fast, fully offline way to sanity-check the
# meson<->cargo wiring below, without pixiewood or an Android toolchain.
gtk_dep = dependency('gtk4')

app_id = 'com.example.Arelm'

# pixiewood.xml's <metainfo> pulls this file in via
# `<xi:include href="build://{arch}/@0@.metainfo.xml"/>` - pixiewood
# resolves that `build:` URI straight into this arch's build directory (and
# will re-run ninja for that specific path first if needed), so the
# metainfo has to actually exist there as a build product, not just as a
# source file. A plain copy is enough here; a real app would use
# import('i18n').merge_file() instead to also merge translations from po/.
configure_file(
  input: 'data' / (app_id + '.metainfo.xml'),
  output: app_id + '.metainfo.xml',
  copy: true,
)

cargo_wrapper = find_program('build-scripts/cargo-build-cdylib.sh')

# Only the real Android cross-build performed by `pixiewood build` (via
# arch/*.cross + android.cross, see gtk-android-builder) needs any of this.
rust_target = ''
build_deps = []
extra_env = environment()

if meson.is_cross_build()
  cpu_family = host_machine.cpu_family()
  if cpu_family == 'aarch64'
    rust_target = 'aarch64-linux-android'
  elif cpu_family == 'x86_64'
    rust_target = 'x86_64-linux-android'
  else
    error('no Rust target triple known for Android ABI: @0@'.format(cpu_family))
  endif

  # `dependency('gtk4')` above resolved to gtk-android-builder's cross-built
  # `gtk` subproject. Grab the actual shared_library() build target that
  # GTK's own meson.build exposes as `libgtk` (alongside the `libgtk_dep` it
  # derives from it and registers via
  # `meson.override_dependency('gtk4', libgtk_dep)`), so we can make ninja
  # build it - and therefore glib/pango/cairo/gdk-pixbuf/etc. too, since
  # libgtk links all of those - *before* it runs us. Without an explicit
  # `depends:` edge here, ninja is free to run our custom_target and GTK's
  # own build in parallel, and cargo's link step would go looking for a
  # libgtk-4.so (and friends) that doesn't exist yet.
  build_deps += subproject('gtk').get_variable('libgtk')

  # meson already resolved the exact per-ABI, per-API-level NDK clang for
  # this cross build (see arch/*.cross + android.cross). Reuse that same
  # compiler as rustc's linker instead of re-deriving the NDK layout
  # ourselves, so this can never drift from what meson/pixiewood actually
  # used to build GTK itself.
  android_cc = meson.get_compiler('c').cmd_array()[0]
  linker_env = 'CARGO_TARGET_' + rust_target.to_upper().replace('-', '_') + '_LINKER'
  extra_env.set(linker_env, android_cc)

  # Subprojects' `pkg.generate()` calls write `*-uninstalled.pc` files (with
  # -L/-I flags pointing straight into the build tree) into this directory
  # as soon as they're configured. This is how the gtk4-sys/glib-sys/etc.
  # `system-deps`-based build scripts of the gtk4-rs crates find headers and
  # libraries for a dependency that pixiewood hasn't `meson install`-ed
  # anywhere on disk yet (that only happens after `ninja` finishes, in
  # pixiewood's own `build` action).
  extra_env.set('PKG_CONFIG_PATH', meson.global_build_root() / 'meson-uninstalled')
  extra_env.set('PKG_CONFIG_ALLOW_CROSS', '1')
endif

# pixiewood's own `prepare` action only ever passes `--buildtype release` or
# leaves the meson default (`debug`); mirror that exactly rather than trying
# to handle every possible buildtype.
profile = get_option('buildtype') == 'release' ? 'release' : 'dev'

libarelm = custom_target('arelm',
  output: 'libarelm.so',
  console: true,
  build_always_stale: true,
  depends: build_deps,
  env: extra_env,
  command: [
    cargo_wrapper,
    '--manifest-path', meson.project_source_root() / 'Cargo.toml',
    '--target-dir', meson.current_build_dir() / 'cargo-target',
    '--out', '@OUTPUT@',
    '--rust-target', rust_target,
    '--profile', profile,
  ],
  install: true,
  install_dir: get_option('libdir'),
  install_tag: 'runtime',
)
