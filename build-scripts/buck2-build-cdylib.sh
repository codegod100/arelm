#!/usr/bin/env bash
# Invoked by meson.build's `custom_target` to build the arelm cdylib with
# buck2 and place the result exactly where meson expects it - the buck2
# equivalent of (and replacement for) the old cargo-build-cdylib.sh.
#
# On a plain (non-cross) `meson setup builddir .`, --rust-target is passed
# empty, which maps to no `--target-platforms=` override at all (root
# .buckconfig's `[parser] target_platform_detector_spec` then resolves
# `//:arelm-lib` to `prelude//platforms:default`, i.e. this host) - same
# "sanity-check the meson<->buck2 wiring with no Android toolchain at hand"
# property the cargo version had (see comments in meson.build). Unlike the
# cargo version, no --manifest-path/--target-dir/PKG_CONFIG_PATH/linker env
# plumbing is needed here: BUCK is always read from the repo root, buck-out/
# is buck2's own fixed output tree, and the NDK linker + PKG_CONFIG_PATH for
# each Android ABI are already hardcoded in toolchains/BUCK and
# third-party/fixups/*/fixups.toml respectively (see those files) - meson
# doesn't need to (and, cross-compiling, mostly can't - see meson.build's
# comment on `dependency('gtk4')`) rediscover any of that itself.
set -euo pipefail

out=""
rust_target=""
profile="dev"

while [ $# -gt 0 ]; do
	case "$1" in
	--out) out="$2"; shift 2 ;;
	--rust-target) rust_target="$2"; shift 2 ;;
	--profile) profile="$2"; shift 2 ;;
	*)
		echo "buck2-build-cdylib.sh: unknown argument: $1" >&2
		exit 1
		;;
	esac
done

if [ -z "$out" ]; then
	echo "buck2-build-cdylib.sh: missing required --out" >&2
	exit 1
fi

# meson passes --out as a path relative to ninja's cwd (the *build* dir) -
# resolve it to absolute before the cd below changes what "relative" means.
case "$out" in
/*) ;; # already absolute
*) out="$PWD/$out" ;;
esac

# ninja runs custom_target commands with the build dir as cwd, not the
# source dir - buck2, unlike the old cargo invocation (which took an
# explicit --manifest-path), always discovers its project root from cwd by
# walking up for a .buckconfig. cd to this script's own parent's parent
# (build-scripts/../ = repo root, where .buckconfig lives) before invoking
# it.
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$script_dir/.."

buck2_args=(build "//:arelm-lib[cdylib]" --show-full-output)

target_platform=""
case "$rust_target" in
"") ;; # host build, no override
aarch64-linux-android) target_platform="//platforms:android-aarch64" ;;
x86_64-linux-android) target_platform="//platforms:android-x86_64" ;;
*)
	echo "buck2-build-cdylib.sh: no buck2 platform() known for rust target '$rust_target'" >&2
	exit 1
	;;
esac
if [ -n "$target_platform" ]; then
	buck2_args+=(--target-platforms "$target_platform")
fi

# Mirrors cargo's --release/dev split: root BUCK reads this same
# `arelm.release` config key (via `read_root_config`) to append
# `-Copt-level=3` &c. to arelm-lib's rustc_flags. Unlike cargo, buck2 has no
# built-in profile concept - a plain `--config` key is the standard way a
# buck2 project threads a caller-chosen setting into a `select()`/
# `read_root_config()` in BUCK files.
if [ "$profile" = "release" ]; then
	buck2_args+=(--config arelm.release=true)
fi

echo "+ buck2 ${buck2_args[*]}" >&2
buck2_output="$(buck2 "${buck2_args[@]}")"

# `buck2 build --show-full-output` prints "<target> <path>" - the built
# cdylib subtarget only ever produces the one line here since we only
# passed the one target above.
built="$(echo "$buck2_output" | awk '{print $2}')"
if [ -z "$built" ] || [ ! -f "$built" ]; then
	echo "buck2-build-cdylib.sh: expected buck2 output missing: ${built:-<empty>}" >&2
	echo "buck2-build-cdylib.sh: buck2 said: $buck2_output" >&2
	exit 1
fi
cp -f "$built" "$out"
