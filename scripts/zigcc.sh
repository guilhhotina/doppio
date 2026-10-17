#!/bin/sh
for arg do
    shift
    case "$arg" in
        --target=*|-target|-Wl,-O1) ;;
        *) set -- "$@" "$arg" ;;
    esac
done
exec zig cc -target "$ZIG_TARGET" ${ZIG_CPU:+-mcpu=$ZIG_CPU} "$@"
