#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
build=${DOPPIO_BUILD:-$(dirname "$root")/doppio-build}/web
dist=$build/dist
emsdk_version=3.1.46
retroarch_version=1.22.2
target=wasm32-unknown-emscripten
toolchain=nightly-2026-10-06
jobs=$(nproc)
mkdir -p "$build"

fetch() {
    [ -e "$2/$3" ] || { mkdir -p "$2"; curl -fsSL "$1" | tar xz -C "$2" --strip-components=1; }
}

use_emsdk() {
    local dir=$build/emsdk
    fetch "https://github.com/emscripten-core/emsdk/archive/refs/tags/$emsdk_version.tar.gz" "$dir" emsdk
    [ -f "$dir/.active-$emsdk_version" ] || {
        "$dir/emsdk" install "$emsdk_version"
        "$dir/emsdk" activate "$emsdk_version"
        touch "$dir/.active-$emsdk_version"
    }
    set +u
    source "$dir/emsdk_env.sh" >/dev/null 2>&1
    set -u
}

core() {
    rustup toolchain install "$toolchain" --profile minimal --component rust-src
    (cd "$root" && RUSTFLAGS="-C target-cpu=mvp -C target-feature=+mutable-globals,+sign-ext" \
        cargo "+$toolchain" rustc -Zbuild-std=std,panic_abort --release --lib --target "$target" --crate-type staticlib)
}

player() {
    local src=$build/retroarch-$retroarch_version
    fetch "https://github.com/libretro/RetroArch/archive/refs/tags/v$retroarch_version.tar.gz" "$src" Makefile.emscripten
    cp "$root/target/$target/release/libdoppio_libretro.a" "$src/libretro_emscripten.a"
    make -C "$src" -f Makefile.emscripten -f <(printf 'LDFLAGS += -lidbfs.js\n') -j"$jobs" LIBRETRO=doppio \
        CC=emcc CXX=em++ LD=emcc AR=emar
    rm -rf "$dist"
    mkdir -p "$dist"
    cp "$src/doppio_libretro.js" "$src/doppio_libretro.wasm" "$root/web/index.html" "$root/web/doppio.game" "$dist/"
    du -sh "$dist"/*
}

use_emsdk
core
player
