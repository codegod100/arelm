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
# meson<->buck2 wiring below, without pixiewood or an Android toolchain.
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

buck2_wrapper = find_program('build-scripts/buck2-build-cdylib.sh')

# Only the real Android cross-build performed by `pixiewood build` (via
# arch/*.cross + android.cross, see gtk-android-builder) needs any of this.
rust_target = ''
build_deps = []

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
  # `depends:` edge here, ninja is free to run our custom_target and buck2's
  # own build in parallel, and buck2's link step would go looking for a
  # libgtk-4.so (and friends) that doesn't exist yet.
  #
  # Unlike the old cargo-based build, buck2 needs no NDK-linker or
  # PKG_CONFIG_PATH environment threaded through from meson here - both are
  # already hardcoded per-ABI in toolchains/BUCK and
  # third-party/fixups/*/fixups.toml (buck2-build-cdylib.sh just maps this
  # rust_target to the matching `//platforms:android-*` and lets those BUCK
  # files take it from there).
  build_deps += subproject('gtk').get_variable('libgtk')
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
  command: [
    buck2_wrapper,
    '--out', '@OUTPUT@',
    '--rust-target', rust_target,
    '--profile', profile,
  ],
  install: true,
  install_dir: get_option('libdir'),
  install_tag: 'runtime',
)
