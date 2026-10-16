#!/bin/sh
set -eu
zssh=${DOPPIO_ZSSH:-/home/guily/work/zedia/scratch/harness/zssh}
"$zssh" 'd=/mnt/usb/sda1/fellow-v8
[ -f /tmp/doppio/pid ] && kill $(cat /tmp/doppio/pid) 2>/dev/null
for p in $(ps | awk "/[f]ellow-v8\/start.sh/ {print \$1}"); do kill $p; done
pids=$(pidof core browser)
[ -n "$pids" ] && kill $pids
i=0
while [ $i -lt 50 ] && pidof core browser >/dev/null; do sleep 0.1; i=$((i + 1)); done
pids=$(pidof core browser) && kill -9 $pids
rm -f /tmp/doppio/pid
cd / && setsid sh $d/start.sh > /dev/null 2>&1 < /dev/null &
sleep 1
ps | grep "[f]ellow-v8/start.sh"'
