#!/bin/sh
set -eu
zssh=${DOPPIO_ZSSH:-/home/guily/work/zedia/scratch/harness/zssh}
"$zssh" 'p=$(cat /tmp/doppio/pid 2>/dev/null)
if [ -n "$p" ] && [ -d /proc/$p ]; then
	echo "pid $p"
	grep -E "^(VmRSS|VmHWM|VmSize|Threads)" /proc/$p/status
else
	echo "core not running"
fi
grep -E "^(MemTotal|MemAvailable)" /proc/meminfo'
