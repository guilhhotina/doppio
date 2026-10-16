#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
mode=${1:-all}
lib=libdoppio_libretro.so
arm=armv7-unknown-linux-gnueabihf
host=x86_64-unknown-linux-gnu

publish() {
    install -D -m 755 "target/$1/release/$lib" "target/out/$2/$lib"
    printf '%-8s %8d bytes  target/out/%s/%s\n' "$2" "$(stat -c %s "target/out/$2/$lib")" "$2" "$lib"
}

build_host() {
    cargo build --release --target "$host"
    publish "$host" x86_64
}

build_arm() {
    rustup target list --installed | grep -qx "$arm" || rustup target add "$arm"
    command -v zig >/dev/null || { echo "zig not found (needed to link armv7)" >&2; exit 1; }
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER="$root/scripts/zigcc-armv7.sh" \
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS="-C target-cpu=cortex-a7 -C target-feature=+neon,+vfp4,+d32" \
        cargo build --release --target "$arm"
    publish "$arm" armv7
}

case "$mode" in
    host) build_host ;;
    arm) build_arm ;;
    all) build_host; build_arm ;;
    *) echo "usage: $0 [host|arm|all]" >&2; exit 2 ;;
esac
