#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
build=${DOPPIO_BUILD:-$(dirname "$root")/doppio-build}
distro=${ZAPER_DISTRO:-$build/zaper-distro}/sources/zedia/v1/opkg
opkg=${DOPPIO_OPKG:-$build/opkg/opkg}
key=${DOPPIO_KEY:-$distro/keys/zedia.key}
plugin=${DOPPIO_PLUGIN:-$build/frontend/armv7/plugin/liblibretro_gecnd.so}
gecnd=${DOPPIO_GECND:-$build/gecnd}
prefix=${DOPPIO_ROOT:-/opt/zedia}
out=${DOPPIO_OUT:-$root/target/out}
core=$root/target/out/armv7/libdoppio_libretro.so
id=tv.zaper.doppio

version() {
    local y d
    IFS=. read -r y d < <(git -C "$1" log -1 --format=%cd --date=format:%Y.%j HEAD)
    printf '1.%d.%d' "$y" "$((10#$d))"
}

[ -n "${DOPPIO_SKIP_BUILD:-}" ] || "$root/scripts/build.sh" arm
[ -f "$core" ] || { echo "missing $core" >&2; exit 1; }
[ -f "$plugin" ] || { echo "missing $plugin: run scripts/frontend-build.sh box" >&2; exit 1; }
[ -x "$opkg" ] || (cd "$distro/program" && CGO_ENABLED=0 go build -o "$opkg" ./cmd/opkg)

stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
mkdir -p "$out" "$stage/zedia" "$stage/libretro/data/plugin" "$stage/doppio/data/lib" "$stage/doppio/data/config"
run() { (cd "$out" && ZEDIA_ROOT_PATH="$stage/zedia" "$opkg" "$@"); }

retro=$(version "$gecnd")
install -m 755 "$plugin" "$stage/libretro/data/plugin/liblibretro_gecnd.so"
cat > "$stage/libretro/control" <<EOF
Package: com.glyengine.libretro
Version: $retro
Architecture: arm
Description: support for emulators in zaper operational system
EOF
run build "$stage/libretro"

install -m 755 "$core" "$stage/doppio/data/lib/libdoppio_libretro.so"
cat > "$stage/doppio/data/config/doppio.toml" <<EOF
[registry.libretro.core]
path = "$prefix/packages/$id/lib"
EOF
cat > "$stage/doppio/control" <<EOF
Package: $id
Version: ${DOPPIO_VERSION:-$(version "$root")}
Architecture: arm
Depends: com.glyengine.libretro (>= $retro)
Description: doppio game
EOF
run key-add -p "$key" >/dev/null
run build -p "$stage/doppio"
