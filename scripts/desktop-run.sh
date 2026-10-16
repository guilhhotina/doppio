#!/bin/sh
set -eu
[ $# -ge 1 ] || { echo "usage: $0 <core.so> [?query#fragment]" >&2; exit 2; }
here=$(cd "$(dirname "$0")" && pwd)
core=$(realpath "$1")
query=${2:-}
build=${DOPPIO_BUILD:-$(dirname "$here")/../doppio-build}
fe=$(realpath "$build/frontend/x86_64")
run=${DOPPIO_DESKTOP_DIR:-/tmp/doppio-desktop}
[ -f "$core" ] || { echo "missing $core" >&2; exit 1; }
[ -x "$fe/core" ] || { echo "missing $fe/core: run scripts/frontend-build.sh" >&2; exit 1; }
mkdir -p "$run"
entry=$(basename "$core")
coredir=$(dirname "$core")
if [ -n "${DOPPIO_PROBE:-}" ]; then
	export PROBE_CORE="$core"
	entry=probe_libretro.so
	coredir=$fe
fi
cat > "$run/doppio.toml" <<EOF
[args]
backend = "${DOPPIO_BACKEND:-gles_glfw}"
window = "${DOPPIO_WINDOW:-1280x720}"
plugins = [
    "$fe/plugin/liblibretro_gecnd.so",
    "$fe/plugin/libkeytest_gecnd.so",
]

[registry.libretro.core]
path = "$coredir"
EOF
cd "$run"
DISPLAY=${DISPLAY:-:0} exec "$fe/core" --conf "$run/doppio.toml" --play "libretro://$entry$query"
