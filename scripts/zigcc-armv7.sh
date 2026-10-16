#!/bin/sh
for arg do
    shift
    case "$arg" in
        --target=*|-target|-Wl,-O1) ;;
        *) set -- "$@" "$arg" ;;
    esac
done
exec zig cc -target arm-linux-gnueabihf.2.17 -mcpu=cortex_a7 "$@"
