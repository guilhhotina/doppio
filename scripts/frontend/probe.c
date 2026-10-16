#include <dlfcn.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#include <libretro.h>

static void *h;

#define FWD(ret, name, params, args) \
    RETRO_API ret name params { static ret (*f) params; if (!f) *(void **)&f = dlsym(h, #name); return f args; }
#define FWDV(name, params, args) \
    RETRO_API void name params { static void (*f) params; if (!f) *(void **)&f = dlsym(h, #name); f args; }

__attribute__((constructor)) static void probe_open(void)
{
    const char *path = getenv("PROBE_CORE");
    h = path ? dlopen(path, RTLD_NOW | RTLD_LOCAL) : NULL;
    fprintf(stderr, "[probe] core=%s handle=%p %s\n", path ? path : "(unset)", h, h ? "" : dlerror());
}

static uint64_t now_us(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000u + (uint64_t)ts.tv_nsec / 1000u;
}

static retro_video_refresh_t real_video;
static unsigned last_w, last_h, video_calls;

static void RETRO_CALLCONV probe_video(const void *data, unsigned w, unsigned hh, size_t pitch)
{
    last_w = w;
    last_h = hh;
    video_calls++;
    real_video(data, w, hh, pitch);
}

FWDV(retro_init, (void), ())
FWDV(retro_deinit, (void), ())
FWD(unsigned, retro_api_version, (void), ())
FWDV(retro_get_system_info, (struct retro_system_info *i), (i))
FWDV(retro_set_environment, (retro_environment_t cb), (cb))
FWDV(retro_set_audio_sample, (retro_audio_sample_t cb), (cb))
FWDV(retro_set_audio_sample_batch, (retro_audio_sample_batch_t cb), (cb))
FWDV(retro_set_input_poll, (retro_input_poll_t cb), (cb))
FWDV(retro_set_input_state, (retro_input_state_t cb), (cb))
FWDV(retro_set_controller_port_device, (unsigned p, unsigned d), (p, d))
FWDV(retro_reset, (void), ())
FWD(size_t, retro_serialize_size, (void), ())
FWD(bool, retro_serialize, (void *d, size_t s), (d, s))
FWD(bool, retro_unserialize, (const void *d, size_t s), (d, s))
FWDV(retro_cheat_reset, (void), ())
FWDV(retro_cheat_set, (unsigned i, bool e, const char *c), (i, e, c))
FWD(bool, retro_load_game, (const struct retro_game_info *g), (g))
FWD(bool, retro_load_game_special, (unsigned t, const struct retro_game_info *g, size_t n), (t, g, n))
FWDV(retro_unload_game, (void), ())
FWD(unsigned, retro_get_region, (void), ())
FWD(void *, retro_get_memory_data, (unsigned id), (id))
FWD(size_t, retro_get_memory_size, (unsigned id), (id))

RETRO_API void retro_set_video_refresh(retro_video_refresh_t cb)
{
    static void (*f)(retro_video_refresh_t);
    if (!f) *(void **)&f = dlsym(h, "retro_set_video_refresh");
    real_video = cb;
    f(probe_video);
}

RETRO_API void retro_get_system_av_info(struct retro_system_av_info *i)
{
    static void (*f)(struct retro_system_av_info *);
    if (!f) *(void **)&f = dlsym(h, "retro_get_system_av_info");
    f(i);
    fprintf(stderr, "[probe] av_info base=%ux%u max=%ux%u aspect=%.3f fps=%.2f rate=%.0f\n",
            i->geometry.base_width, i->geometry.base_height, i->geometry.max_width, i->geometry.max_height,
            (double)i->geometry.aspect_ratio, i->timing.fps, i->timing.sample_rate);
}

RETRO_API void retro_run(void)
{
    static void (*f)(void);
    static uint64_t window_start, prev;
    static uint64_t min_dt = UINT64_MAX, max_dt, cost_sum, cost_max;
    static unsigned calls;
    if (!f) *(void **)&f = dlsym(h, "retro_run");
    uint64_t t0 = now_us();
    if (!window_start) window_start = t0;
    if (prev) {
        uint64_t dt = t0 - prev;
        min_dt = dt < min_dt ? dt : min_dt;
        max_dt = dt > max_dt ? dt : max_dt;
    }
    prev = t0;
    f();
    uint64_t cost = now_us() - t0;
    cost_sum += cost;
    cost_max = cost > cost_max ? cost : cost_max;
    calls++;
    if (t0 - window_start >= 2000000u) {
        double secs = (double)(t0 - window_start) / 1e6;
        fprintf(stderr, "[probe] retro_run %.2f Hz (%u in %.2fs) dt min=%.1fms max=%.1fms run avg=%.2fms max=%.2fms video=%u last=%ux%u\n",
                calls / secs, calls, secs, min_dt / 1000.0, max_dt / 1000.0,
                cost_sum / 1000.0 / calls, cost_max / 1000.0, video_calls, last_w, last_h);
        window_start = t0;
        calls = 0;
        video_calls = 0;
        cost_sum = 0;
        cost_max = 0;
        min_dt = UINT64_MAX;
        max_dt = 0;
    }
}
