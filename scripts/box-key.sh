#!/bin/sh
set -eu
usage() { echo "usage: $0 <tap|down|up> <a|b|c|d|e|f|up|down|left|right> [port]" >&2; exit 2; }
[ $# -ge 2 ] || usage
case $1 in tap|down|up) ;; *) usage ;; esac
case $2 in a|b|c|d|e|f|up|down|left|right) ;; *) usage ;; esac
port=${3:-0}
zssh=${DOPPIO_ZSSH:-/home/guily/work/zedia/scratch/harness/zssh}
"$zssh" "[ -p /tmp/zedia-keys ] && echo '$1 $2 $port' > /tmp/zedia-keys"
