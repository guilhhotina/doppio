#!/bin/sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)
mode=${1:-all}
build=${DOPPIO_BUILD:-$(dirname "$here")/../doppio-build}
distro=${ZAPER_DISTRO:-/home/guily/work/zedia/zaper-distro}
rev=f37f100560d2ae1fbf390e477ba4aa7c71126637
jobs=$(nproc)
mkdir -p "$build"
build=$(realpath "$build")
command -v xxd >/dev/null || { echo "xxd missing (nicky add tinyxxd)" >&2; exit 1; }

gecnd_src() {
	[ -d "$build/gecnd/.git" ] || git clone -q --branch feature/libretro https://github.com/gly-engine/core-native-desktop.git "$build/gecnd"
	git -C "$build/gecnd" cat-file -e "$rev^{commit}" 2>/dev/null || git -C "$build/gecnd" fetch -q origin feature/libretro
	git -C "$build/gecnd" checkout -q "$rev"
}

desktop() {
	gecnd_src
	src=$build/gecnd
	out=$src/build_desktop
	libs=
	[ -e /usr/include/sys/capability.h ] && libs=-lcap
	cmake -S "$src" -B "$out" -DCMAKE_BUILD_TYPE=RelWithDebInfo \
		-DGECND_USE_PLUGIN_LIBRETRO=ON -DGECND_USE_FFI=ON -DGECND_USE_LUAJIT=ON \
		-DGECND_USE_FFMPEG=OFF -DGECND_USE_GL_SDL2=OFF \
		-DCMAKE_C_STANDARD_LIBRARIES="$libs"
	make -C "$out" -j"$jobs" core libretro_gecnd
	fe=$build/frontend/x86_64
	mkdir -p "$fe/plugin"
	cp "$out/bin/core" "$fe/core"
	cp "$out/lib/liblibretro_gecnd.so" "$fe/plugin/"
	cc -O2 -shared -fPIC -I"$src/include" -o "$fe/plugin/libkeytest_gecnd.so" "$here/frontend/keytest.c" -lpthread
	cc -O2 -shared -fPIC -I"$src/vendor/mono/retroarch" -o "$fe/probe_libretro.so" "$here/frontend/probe.c" -ldl
	ls -l "$fe" "$fe/plugin"
}

box() {
	gecnd_src
	wt=$build/zaper-distro
	if [ ! -d "$wt" ]; then
		git -C "$distro" worktree add --detach "$wt" origin/develop
		[ -d "$distro/vendor" ] && cp -a --reflink=auto "$distro/vendor" "$wt/vendor"
	fi
	vend=$wt/vendor/gamely/core-native-desktop
	if [ "$(git -C "$vend" rev-parse HEAD 2>/dev/null)" != "$rev" ]; then
		rm -rf "$vend"
		git clone -q "$build/gecnd" "$vend"
		git -C "$vend" checkout -q "$rev"
	fi
	sed -i "s/^set(GECND_VERSION .*/set(GECND_VERSION \"$rev\")/" "$wt/sources/opensource/v1/gly-engine/cmake/core-native-desktop.cmake"
	out=$wt/build_alitech_aui
	lws=$out/_deps/gecnd-build/libwebsockets_proj-prefix/src/libwebsockets_proj-build
	mkdir -p "$lws"
	[ -f "$lws/CMakeCache.txt" ] || echo 'LWS_WITH_LIBCAP:BOOL=OFF' > "$lws/CMakeCache.txt"
	(cd "$wt" && cmake --preset ali -DZEDIA_USE_LIBRETRO=ON \
		-DGECND_USE_GL_GLFW=OFF -DGECND_USE_GL_SDL2=OFF \
		-DCMAKE_LINK_DEPENDS_USE_LINKER=OFF -DCMAKE_IGNORE_PATH=/usr/lib32 \
		-DMATH_LIBRARY=m -DRT_LIBRARY=rt)
	cmake --build "$out" -j"$jobs" --target core libretro_gecnd alitech_gecnd
	fe=$build/frontend/armv7
	mkdir -p "$fe/plugin"
	llvm-strip -o "$fe/core" "$out/bin/core"
	llvm-strip -o "$fe/plugin/liblibretro_gecnd.so" "$out/lib/liblibretro_gecnd.so"
	llvm-strip -o "$fe/plugin/libalitech_gecnd.so" "$out/lib/libalitech_gecnd.so"
	"$out/zig-cc" -shared -fPIC -s -I"$vend/include" -o "$fe/plugin/libkeytest_gecnd.so" "$here/frontend/keytest.c" -lpthread
	"$out/zig-cc" -shared -fPIC -s -I"$wt/vendor/mono/retroarch" -o "$fe/probe_libretro.so" "$here/frontend/probe.c" -ldl
	ls -l "$fe" "$fe/plugin"
}

case $mode in
	desktop) desktop ;;
	box) box ;;
	all) desktop; box ;;
	*) echo "usage: $0 [desktop|box|all]" >&2; exit 2 ;;
esac
