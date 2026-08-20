#!/usr/bin/env bash
# Invoked by meson.build's `custom_target` to build the arelm cdylib with
# cargo and place the result exactly where meson expects it.
#
# On a plain (non-cross) `meson setup builddir .`, --rust-target is passed
# empty and this just builds for the host - which is what makes it possible
# to sanity-check this whole meson<->cargo wiring without pixiewood or an
# Android toolchain at hand (see comments in meson.build).
set -euo pipefail

manifest_path=""
target_dir=""
out=""
rust_target=""
profile="dev"

while [ $# -gt 0 ]; do
	case "$1" in
	--manifest-path) manifest_path="$2"; shift 2 ;;
	--target-dir) target_dir="$2"; shift 2 ;;
	--out) out="$2"; shift 2 ;;
	--rust-target) rust_target="$2"; shift 2 ;;
	--profile) profile="$2"; shift 2 ;;
	*)
		echo "cargo-build-cdylib.sh: unknown argument: $1" >&2
		exit 1
		;;
	esac
done

for required in manifest_path target_dir out; do
	if [ -z "${!required}" ]; then
		echo "cargo-build-cdylib.sh: missing required --${required//_/-}" >&2
		exit 1
	fi
done

cargo_args=(build --package arelm --lib --manifest-path "$manifest_path" --target-dir "$target_dir")
profile_dir="debug"
if [ "$profile" = "release" ]; then
	cargo_args+=(--release)
	profile_dir="release"
fi

out_subdir="$target_dir/$profile_dir"
if [ -n "$rust_target" ]; then
	if ! rustc --print target-list | grep -qx "$rust_target"; then
		echo "cargo-build-cdylib.sh: rustc doesn't know target '$rust_target'" >&2
		exit 1
	fi
	if ! rustup target list --installed 2>/dev/null | grep -qx "$rust_target"; then
		echo "cargo-build-cdylib.sh: note: '$rust_target' does not look like an installed rustup target;" >&2
		echo "cargo-build-cdylib.sh: run: rustup target add $rust_target" >&2
	fi
	cargo_args+=(--target "$rust_target")
	out_subdir="$target_dir/$rust_target/$profile_dir"
fi

echo "+ cargo ${cargo_args[*]}" >&2
cargo "${cargo_args[@]}"

built="$out_subdir/libarelm.so"
if [ ! -f "$built" ]; then
	echo "cargo-build-cdylib.sh: expected cargo output missing: $built" >&2
	exit 1
fi
cp -f "$built" "$out"
