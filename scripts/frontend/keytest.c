#include <pthread.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#include <gecnd.h>

typedef void (*push_name_fn)(const char *name, bool pressed, int port, uint32_t ttl_ms);
static push_name_fn g_push;

static void apply(const char *cmd, const char *name, int port)
{
    if (!strcmp(cmd, "tap")) {
        g_push(name, true, port, 0);
        usleep(120000);
        g_push(name, false, port, 0);
    } else if (!strcmp(cmd, "down")) {
        g_push(name, true, port, 0);
    } else if (!strcmp(cmd, "up")) {
        g_push(name, false, port, 0);
    }
}

static void *reader(void *arg)
{
    (void)arg;
    const char *path = getenv("ZEDIA_KEYS");
    if (!path || !path[0]) path = "/tmp/zedia-keys";
    unlink(path);
    if (mkfifo(path, 0600) != 0) return NULL;
    for (;;) {
        FILE *f = fopen(path, "r");
        if (!f) return NULL;
        char line[96];
        while (fgets(line, sizeof(line), f)) {
            char cmd[16], name[16];
            int port = 0;
            if (sscanf(line, "%15s %15s %d", cmd, name, &port) < 2) continue;
            fprintf(stderr, "[keytest] %s %s %d\n", cmd, name, port);
            apply(cmd, name, port);
        }
        fclose(f);
    }
    return NULL;
}

void coreopen_keytest_gecnd(gecnd_plugin_t *plugin)
{
    gecnd_api_t *api = plugin->require("v1");
    api->registry("get", "function:gamely_daemon_input_push_name", &g_push, NULL);
    if (!g_push) {
        fprintf(stderr, "[keytest] push_name unavailable\n");
        return;
    }
    pthread_t t;
    if (pthread_create(&t, NULL, reader, NULL) == 0) pthread_detach(t);
    fprintf(stderr, "[keytest] ready\n");
}
