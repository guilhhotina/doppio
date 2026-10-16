#!/bin/sh
set -eu
[ $# -ge 1 ] || { echo "usage: $0 <core.so> [?query#fragment] [seconds]" >&2; exit 2; }
here=$(cd "$(dirname "$0")" && pwd)
core=$1
query=${2:-}
secs=${3:-0}
build=${DOPPIO_BUILD:-$(dirname "$here")/../doppio-build}
fe=$build/frontend/armv7
zssh=${DOPPIO_ZSSH:-/home/guily/work/zedia/scratch/harness/zssh}
name=$(basename "$core")
[ -f "$core" ] || { echo "missing $core" >&2; exit 1; }
[ -x "$fe/core" ] || { echo "missing $fe/core: run scripts/frontend-build.sh" >&2; exit 1; }

stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
cp -r "$fe/." "$stage/"
cp "$core" "$stage/$name"
entry=$name
probe=
if [ -n "${DOPPIO_PROBE:-}" ]; then
	entry=probe_libretro.so
	probe="PROBE_CORE=/tmp/doppio/$name"
fi
bin=./core
alitech=/tmp/doppio/plugin/libalitech_gecnd.so
if [ -n "${DOPPIO_IMAGE_CORE:-}" ]; then
	bin=/opt/zedia/core
	alitech=/opt/zedia/plugin/libalitech_gecnd.so
fi
cat > "$stage/doppio.toml" <<EOF
[envs]
DISABLE_ALSA = "YES"
cfg_resolution = "1920x1080"

[args]
screen = "1920x1080"
plugins = [
    "$alitech",
    "/tmp/doppio/plugin/liblibretro_gecnd.so",
    "/tmp/doppio/plugin/libkeytest_gecnd.so",
]

[registry.libretro.core]
path = "/tmp/doppio"
EOF

"$zssh" 'for p in $(ps | awk "/[f]ellow-v8\/start.sh/ {print \$1}"); do kill $p; done
pids=$(pidof core browser)
[ -n "$pids" ] && kill $pids
i=0
while [ $i -lt 50 ] && pidof core browser >/dev/null; do sleep 0.1; i=$((i + 1)); done
pids=$(pidof core browser) && kill -9 $pids
rm -rf /tmp/doppio && mkdir -p /tmp/doppio'
tar -C "$stage" -cf - . | "$zssh" 'tar -C /tmp/doppio -xf -'
"$zssh" "cd /tmp/doppio && chmod 755 core && (env $probe setsid $bin --conf ./doppio.toml ${DOPPIO_ARGS:-} --play 'libretro://$entry$query' > /tmp/doppio/log 2>&1 < /dev/null & echo \$! > /tmp/doppio/pid)
[ $secs -gt 0 ] && (setsid sh -c 'sleep $secs; kill \$(cat /tmp/doppio/pid) 2>/dev/null' > /dev/null 2>&1 < /dev/null &)
sleep 4
echo pid \$(cat /tmp/doppio/pid)
grep -v Fontconfig /tmp/doppio/log"
