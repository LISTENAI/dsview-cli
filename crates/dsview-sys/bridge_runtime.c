#include "wrapper.h"

#if defined(_WIN32)
#include <windows.h>
#else
#include <dlfcn.h>
#endif
#include <glib.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "libsigrok-internal.h"

typedef int (*ds_lib_init_fn)(void);
typedef int (*ds_lib_exit_fn)(void);
typedef void (*ds_set_firmware_resource_dir_fn)(const char *dir);
typedef int (*ds_get_device_list_fn)(struct ds_device_base_info **out_list, int *out_count);
typedef int (*ds_active_device_fn)(ds_device_handle handle);
typedef int (*ds_release_actived_device_fn)(void);
typedef int (*ds_get_last_error_fn)(void);
typedef int (*ds_get_actived_device_init_status_fn)(int *status);
typedef int (*ds_get_actived_device_config_fn)(const struct sr_channel *ch, const struct sr_channel_group *cg, int key, GVariant **data);
typedef int (*ds_get_actived_device_config_list_fn)(const struct sr_channel_group *cg, int key, GVariant **data);
typedef int (*ds_set_actived_device_config_fn)(const struct sr_channel *ch, const struct sr_channel_group *cg, int key, GVariant *data);
typedef int (*ds_set_event_callback_fn)(void *cb);
typedef int (*ds_set_datafeed_callback_fn)(void *cb);
typedef int (*ds_start_collect_fn)(void);
typedef int (*ds_stop_collect_fn)(void);
typedef int (*ds_is_collecting_fn)(void);
typedef int (*ds_enable_device_channel_index_fn)(int channel_index, gboolean enable);
typedef const struct sr_output_module *(*sr_output_find_fn)(char *id);
typedef const struct sr_output *(*sr_output_new_fn)(const struct sr_output_module *omod, GHashTable *options, const struct sr_dev_inst *sdi);
typedef int (*sr_output_send_fn)(const struct sr_output *o, const struct sr_datafeed_packet *packet, GString **out);
typedef int (*sr_output_free_fn)(const struct sr_output *o);

typedef void (*dslib_event_callback_t)(int event);
typedef void (*ds_datafeed_callback_t)(const struct sr_dev_inst *sdi, const struct sr_datafeed_packet *packet);

enum dsview_export_packet_type {
    DSVIEW_EXPORT_PACKET_META = 1,
    DSVIEW_EXPORT_PACKET_LOGIC = 2,
    DSVIEW_EXPORT_PACKET_END = 3,
};

enum dsview_bridge_export_status {
    DSVIEW_EXPORT_OK = 0,
    DSVIEW_EXPORT_ERR_GENERIC = -100,
    DSVIEW_EXPORT_ERR_NO_STREAM = -101,
    DSVIEW_EXPORT_ERR_OVERFLOW = -102,
    DSVIEW_EXPORT_ERR_BAD_END_STATUS = -103,
    DSVIEW_EXPORT_ERR_MISSING_SAMPLERATE = -104,
    DSVIEW_EXPORT_ERR_NO_ENABLED_CHANNELS = -105,
    DSVIEW_EXPORT_ERR_OUTPUT_MODULE = -106,
    DSVIEW_EXPORT_ERR_RUNTIME = -107,
};

struct dsview_bridge_api {
    void *library_handle;
    ds_lib_init_fn ds_lib_init;
    ds_lib_exit_fn ds_lib_exit;
    ds_set_event_callback_fn ds_set_event_callback;
    ds_set_datafeed_callback_fn ds_set_datafeed_callback;
    ds_start_collect_fn ds_start_collect;
    ds_stop_collect_fn ds_stop_collect;
    ds_is_collecting_fn ds_is_collecting;
    ds_set_firmware_resource_dir_fn ds_set_firmware_resource_dir;
    ds_get_device_list_fn ds_get_device_list;
    ds_active_device_fn ds_active_device;
    ds_release_actived_device_fn ds_release_actived_device;
    ds_get_last_error_fn ds_get_last_error;
    ds_get_actived_device_init_status_fn ds_get_actived_device_init_status;
    ds_get_actived_device_config_fn ds_get_actived_device_config;
    ds_get_actived_device_config_list_fn ds_get_actived_device_config_list;
    ds_set_actived_device_config_fn ds_set_actived_device_config;
    ds_enable_device_channel_index_fn ds_enable_device_channel_index;
    sr_output_find_fn sr_output_find;
    sr_output_new_fn sr_output_new;
    sr_output_send_fn sr_output_send;
    sr_output_free_fn sr_output_free;
    char last_error[512];
};

struct dsview_retained_packet {
    int type;
    int status;
    unsigned long long samplerate_hz;
    size_t length;
    uint16_t unitsize;
    uint16_t data_error;
    uint64_t error_pattern;
    uint8_t *data;
};

struct dsview_recorded_stream {
    struct dsview_retained_packet *packets;
    size_t packet_count;
    size_t packet_capacity;
    size_t payload_bytes;
    size_t payload_capacity;
    int overflowed;
    int saw_logic_packet;
    int saw_end_packet;
    int end_packet_status;
    unsigned long long samplerate_hz;
    int has_samplerate;
    unsigned long long sample_count;
    uint16_t max_unitsize;
    uint16_t expected_unitsize;
};

static struct dsview_bridge_api g_bridge_api;
static struct dsview_bridge_acquisition_summary g_acquisition_summary;
static int g_acquisition_callback_registration_active = 0;
static struct dsview_recorded_stream g_recorded_stream;

static void dsview_bridge_clear_registered_callbacks(void);
static void dsview_bridge_reset_recorded_stream(void);
static int dsview_bridge_prepare_recording_capacity(void);
static int dsview_bridge_record_packet(const struct sr_datafeed_packet *packet);
static int dsview_bridge_export_stream(const struct dsview_vcd_export_request *request, const struct dsview_recorded_stream *stream, struct dsview_export_buffer *out_buffer);
static int dsview_bridge_build_vcd_device(const struct dsview_vcd_export_request *request, struct sr_dev_inst **out_sdi);
static void dsview_bridge_free_vcd_device(struct sr_dev_inst *sdi);
static int dsview_bridge_emit_packet(const struct sr_output *output, const struct dsview_retained_packet *packet, GString **assembled_output);
static uint16_t dsview_bridge_expected_logic_unitsize(void);

static void dsview_bridge_set_error_from_text(const char *message)
{
    if (message == NULL) {
        g_bridge_api.last_error[0] = '\0';
        return;
    }

    strncpy(g_bridge_api.last_error, message, sizeof(g_bridge_api.last_error) - 1);
    g_bridge_api.last_error[sizeof(g_bridge_api.last_error) - 1] = '\0';
}

#if defined(_WIN32)
static void dsview_bridge_clear_loader_error(void)
{
    SetLastError(ERROR_SUCCESS);
}

static void dsview_bridge_set_error_from_loader(void)
{
    DWORD error = GetLastError();
    char buffer[128];

    if (error == ERROR_SUCCESS) {
        dsview_bridge_set_error_from_text("unknown dynamic loader error");
        return;
    }

    snprintf(buffer, sizeof(buffer), "dynamic loader error code %lu", (unsigned long)error);
    dsview_bridge_set_error_from_text(buffer);
}

static void *dsview_bridge_dlopen(const char *path)
{
    return (void *)LoadLibraryA(path);
}

static void *dsview_bridge_dlsym(void *library_handle, const char *name)
{
    return (void *)GetProcAddress((HMODULE)library_handle, name);
}

static void dsview_bridge_dlclose(void *library_handle)
{
    FreeLibrary((HMODULE)library_handle);
}
#else
static void dsview_bridge_clear_loader_error(void)
{
    dlerror();
}

static void dsview_bridge_set_error_from_loader(void)
{
    const char *error = dlerror();
    dsview_bridge_set_error_from_text(error != NULL ? error : "unknown dynamic loader error");
}

static void *dsview_bridge_dlopen(const char *path)
{
    return dlopen(path, RTLD_NOW | RTLD_LOCAL);
}

static void *dsview_bridge_dlsym(void *library_handle, const char *name)
{
    return dlsym(library_handle, name);
}

static void dsview_bridge_dlclose(void *library_handle)
{
    dlclose(library_handle);
}
#endif

static void *dsview_bridge_load_symbol(const char *name, int *status_out)
{
    void *symbol = NULL;

    dsview_bridge_clear_loader_error();
    symbol = dsview_bridge_dlsym(g_bridge_api.library_handle, name);
    if (symbol == NULL) {
        dsview_bridge_set_error_from_loader();
        if (status_out != NULL) {
            *status_out = DSVIEW_BRIDGE_ERR_DLSYM;
        }
    }

    return symbol;
}

static int dsview_bridge_get_uint64_config(int key, unsigned long long *value)
{
    GVariant *data = NULL;
    int status;

    if (value == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (g_bridge_api.ds_get_actived_device_config == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    status = g_bridge_api.ds_get_actived_device_config(NULL, NULL, key, &data);
    if (status != SR_OK) {
        return status;
    }
    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    *value = g_variant_get_uint64(data);
    g_variant_unref(data);
    return SR_OK;
}

static int dsview_bridge_get_int16_config(int key, int *value)
{
    GVariant *data = NULL;
    int status;

    if (value == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (g_bridge_api.ds_get_actived_device_config == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    status = g_bridge_api.ds_get_actived_device_config(NULL, NULL, key, &data);
    if (status != SR_OK) {
        return status;
    }
    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    *value = g_variant_get_int16(data);
    g_variant_unref(data);
    return SR_OK;
}

static int dsview_bridge_get_double_config(int key, double *value)
{
    GVariant *data = NULL;
    int status;

    if (value == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (g_bridge_api.ds_get_actived_device_config == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    status = g_bridge_api.ds_get_actived_device_config(NULL, NULL, key, &data);
    if (status != SR_OK) {
        return status;
    }
    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    *value = g_variant_get_double(data);
    g_variant_unref(data);
    return SR_OK;
}

static unsigned short dsview_bridge_mode_max_enabled_channels(const char *name)
{
    const char *marker;
    char *end = NULL;
    unsigned long parsed;

    if (name == NULL) {
        return 0;
    }

    marker = strrchr(name, 'x');
    if (marker == NULL || marker[1] == '\0') {
        return 0;
    }

    parsed = strtoul(marker + 1, &end, 10);
    if (end == marker + 1 || (end != NULL && *end != '\0') || parsed > 0xffffUL) {
        return 0;
    }

    return (unsigned short)parsed;
}

static void dsview_bridge_free_retained_packet(struct dsview_retained_packet *packet)
{
    if (packet == NULL) {
        return;
    }

    if (packet->data != NULL) {
        free(packet->data);
    }
    memset(packet, 0, sizeof(*packet));
}

static void dsview_bridge_reset_recorded_stream(void)
{
    size_t index;

    if (g_recorded_stream.packets != NULL) {
        for (index = 0; index < g_recorded_stream.packet_count; index++) {
            dsview_bridge_free_retained_packet(&g_recorded_stream.packets[index]);
        }
        free(g_recorded_stream.packets);
    }

    memset(&g_recorded_stream, 0, sizeof(g_recorded_stream));
    g_recorded_stream.end_packet_status = -1;
}

static uint16_t dsview_bridge_expected_logic_unitsize(void)
{
    return g_recorded_stream.expected_unitsize != 0 ? g_recorded_stream.expected_unitsize : 1;
}

static int dsview_bridge_prepare_recording_capacity(void)
{
    unsigned long long sample_limit = 0;
    int valid_channel_count = 0;
    size_t unitsize;
    size_t packet_capacity;
    size_t payload_capacity;

    dsview_bridge_reset_recorded_stream();

    if (dsview_bridge_get_uint64_config(SR_CONF_LIMIT_SAMPLES, &sample_limit) != SR_OK) {
        return DSVIEW_EXPORT_ERR_RUNTIME;
    }
    if (dsview_bridge_get_int16_config(SR_CONF_VLD_CH_NUM, &valid_channel_count) != SR_OK) {
        return DSVIEW_EXPORT_ERR_RUNTIME;
    }
    if (sample_limit == 0 || valid_channel_count <= 0) {
        return DSVIEW_EXPORT_ERR_NO_ENABLED_CHANNELS;
    }

    unitsize = (size_t)((valid_channel_count + 7) / 8);
    if (unitsize == 0) {
        unitsize = 1;
    }

    if (sample_limit > (unsigned long long)(SIZE_MAX / unitsize)) {
        return DSVIEW_EXPORT_ERR_OVERFLOW;
    }

    payload_capacity = (size_t)sample_limit * unitsize;
    if (sample_limit > (unsigned long long)(SIZE_MAX - 8)) {
        return DSVIEW_EXPORT_ERR_OVERFLOW;
    }
    packet_capacity = (size_t)sample_limit + 8;

    g_recorded_stream.packets = calloc(packet_capacity, sizeof(*g_recorded_stream.packets));
    if (g_recorded_stream.packets == NULL) {
        return SR_ERR_MALLOC;
    }

    g_recorded_stream.packet_capacity = packet_capacity;
    g_recorded_stream.payload_capacity = payload_capacity;
    g_recorded_stream.expected_unitsize = (uint16_t)unitsize;
    return SR_OK;
}

static int dsview_bridge_append_retained_packet(const struct dsview_retained_packet *packet)
{
    if (g_recorded_stream.overflowed) {
        return DSVIEW_EXPORT_ERR_OVERFLOW;
    }
    if (g_recorded_stream.packet_count >= g_recorded_stream.packet_capacity) {
        g_recorded_stream.overflowed = 1;
        return DSVIEW_EXPORT_ERR_OVERFLOW;
    }

    g_recorded_stream.packets[g_recorded_stream.packet_count++] = *packet;
    return SR_OK;
}

static int dsview_bridge_record_meta_packet(const struct sr_datafeed_packet *packet)
{
    const struct sr_datafeed_meta *meta = packet->payload;
    const struct sr_config *src;
    GSList *item;
    struct dsview_retained_packet retained;

    if (meta == NULL) {
        return SR_OK;
    }

    memset(&retained, 0, sizeof(retained));
    retained.type = DSVIEW_EXPORT_PACKET_META;
    retained.status = packet->status;

    for (item = meta->config; item != NULL; item = item->next) {
        src = item->data;
        if (src == NULL || src->key != SR_CONF_SAMPLERATE || src->data == NULL) {
            continue;
        }
        retained.samplerate_hz = g_variant_get_uint64(src->data);
        if (retained.samplerate_hz != 0) {
            g_recorded_stream.samplerate_hz = retained.samplerate_hz;
            g_recorded_stream.has_samplerate = 1;
        }
    }

    return dsview_bridge_append_retained_packet(&retained);
}

static int dsview_bridge_record_logic_packet(const struct sr_datafeed_packet *packet)
{
    const struct sr_datafeed_logic *logic = packet->payload;
    struct dsview_retained_packet retained;
    unsigned long long packet_samples;
    uint16_t unitsize;

    if (logic == NULL || logic->data == NULL || logic->length == 0) {
        return DSVIEW_EXPORT_ERR_GENERIC;
    }

    unitsize = dsview_bridge_expected_logic_unitsize();
    if (logic->unitsize != 0 && logic->format != LA_CROSS_DATA) {
        unitsize = logic->unitsize;
    }
    if ((logic->length % unitsize) != 0) {
        return DSVIEW_EXPORT_ERR_GENERIC;
    }
    if (logic->length > SIZE_MAX) {
        return DSVIEW_EXPORT_ERR_OVERFLOW;
    }
    if (g_recorded_stream.payload_bytes > g_recorded_stream.payload_capacity - (size_t)logic->length) {
        g_recorded_stream.overflowed = 1;
        return DSVIEW_EXPORT_ERR_OVERFLOW;
    }

    memset(&retained, 0, sizeof(retained));
    retained.type = DSVIEW_EXPORT_PACKET_LOGIC;
    retained.status = packet->status;
    retained.length = (size_t)logic->length;
    retained.unitsize = unitsize;
    retained.data_error = logic->data_error;
    retained.error_pattern = logic->error_pattern;
    retained.data = malloc(retained.length);
    if (retained.data == NULL) {
        return SR_ERR_MALLOC;
    }
    memcpy(retained.data, logic->data, retained.length);

    packet_samples = logic->length / unitsize;
    if (g_recorded_stream.sample_count > G_MAXUINT64 - packet_samples) {
        free(retained.data);
        g_recorded_stream.overflowed = 1;
        return DSVIEW_EXPORT_ERR_OVERFLOW;
    }

    g_recorded_stream.payload_bytes += retained.length;
    g_recorded_stream.sample_count += packet_samples;
    if (retained.unitsize > g_recorded_stream.max_unitsize) {
        g_recorded_stream.max_unitsize = retained.unitsize;
    }
    g_recorded_stream.saw_logic_packet = 1;

    return dsview_bridge_append_retained_packet(&retained);
}

static int dsview_bridge_record_end_packet(const struct sr_datafeed_packet *packet)
{
    struct dsview_retained_packet retained;

    memset(&retained, 0, sizeof(retained));
    retained.type = DSVIEW_EXPORT_PACKET_END;
    retained.status = packet->status;
    g_recorded_stream.saw_end_packet = 1;
    g_recorded_stream.end_packet_status = packet->status;
    return dsview_bridge_append_retained_packet(&retained);
}

static int dsview_bridge_record_packet(const struct sr_datafeed_packet *packet)
{
    if (packet == NULL || g_recorded_stream.overflowed) {
        return g_recorded_stream.overflowed ? DSVIEW_EXPORT_ERR_OVERFLOW : SR_OK;
    }

    switch (packet->type) {
    case SR_DF_META:
        return dsview_bridge_record_meta_packet(packet);
    case SR_DF_LOGIC:
        return dsview_bridge_record_logic_packet(packet);
    case SR_DF_END:
        return dsview_bridge_record_end_packet(packet);
    default:
        return SR_OK;
    }
}

int dsview_bridge_load_library(const char *path)
{
    int status = DSVIEW_BRIDGE_OK;

    if (path == NULL || path[0] == '\0') {
        dsview_bridge_set_error_from_text("library path must not be empty");
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    dsview_bridge_unload_library();

    dsview_bridge_clear_loader_error();
    g_bridge_api.library_handle = dsview_bridge_dlopen(path);
    if (g_bridge_api.library_handle == NULL) {
        dsview_bridge_set_error_from_loader();
        return DSVIEW_BRIDGE_ERR_DLOPEN;
    }

    g_bridge_api.ds_lib_init = (ds_lib_init_fn)dsview_bridge_load_symbol("ds_lib_init", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_lib_exit = (ds_lib_exit_fn)dsview_bridge_load_symbol("ds_lib_exit", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_set_event_callback =
        (ds_set_event_callback_fn)dsview_bridge_load_symbol("ds_set_event_callback", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_set_datafeed_callback =
        (ds_set_datafeed_callback_fn)dsview_bridge_load_symbol("ds_set_datafeed_callback", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_start_collect =
        (ds_start_collect_fn)dsview_bridge_load_symbol("ds_start_collect", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_stop_collect =
        (ds_stop_collect_fn)dsview_bridge_load_symbol("ds_stop_collect", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_is_collecting =
        (ds_is_collecting_fn)dsview_bridge_load_symbol("ds_is_collecting", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_set_firmware_resource_dir =
        (ds_set_firmware_resource_dir_fn)dsview_bridge_load_symbol("ds_set_firmware_resource_dir", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_get_device_list =
        (ds_get_device_list_fn)dsview_bridge_load_symbol("ds_get_device_list", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_active_device =
        (ds_active_device_fn)dsview_bridge_load_symbol("ds_active_device", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_release_actived_device =
        (ds_release_actived_device_fn)dsview_bridge_load_symbol("ds_release_actived_device", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_get_last_error =
        (ds_get_last_error_fn)dsview_bridge_load_symbol("ds_get_last_error", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_get_actived_device_init_status =
        (ds_get_actived_device_init_status_fn)dsview_bridge_load_symbol("ds_get_actived_device_init_status", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_get_actived_device_config =
        (ds_get_actived_device_config_fn)dsview_bridge_load_symbol("ds_get_actived_device_config", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_get_actived_device_config_list =
        (ds_get_actived_device_config_list_fn)dsview_bridge_load_symbol("ds_get_actived_device_config_list", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_set_actived_device_config =
        (ds_set_actived_device_config_fn)dsview_bridge_load_symbol("ds_set_actived_device_config", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.ds_enable_device_channel_index =
        (ds_enable_device_channel_index_fn)dsview_bridge_load_symbol("ds_enable_device_channel_index", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.sr_output_find =
        (sr_output_find_fn)dsview_bridge_load_symbol("sr_output_find", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.sr_output_new =
        (sr_output_new_fn)dsview_bridge_load_symbol("sr_output_new", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.sr_output_send =
        (sr_output_send_fn)dsview_bridge_load_symbol("sr_output_send", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    g_bridge_api.sr_output_free =
        (sr_output_free_fn)dsview_bridge_load_symbol("sr_output_free", &status);
    if (status != DSVIEW_BRIDGE_OK) {
        dsview_bridge_unload_library();
        return status;
    }

    dsview_bridge_set_error_from_text(NULL);
    return DSVIEW_BRIDGE_OK;
}

void dsview_bridge_unload_library(void)
{
    dsview_bridge_clear_registered_callbacks();
    dsview_bridge_reset_recorded_stream();

    if (g_bridge_api.library_handle != NULL) {
        dsview_bridge_dlclose(g_bridge_api.library_handle);
    }

    memset(&g_bridge_api, 0, sizeof(g_bridge_api));
    memset(&g_acquisition_summary, 0, sizeof(g_acquisition_summary));
}

int dsview_bridge_is_loaded(void)
{
    return g_bridge_api.library_handle != NULL;
}

const char *dsview_bridge_last_loader_error(void)
{
    return g_bridge_api.last_error;
}

int dsview_bridge_ds_lib_init(void)
{
    if (g_bridge_api.ds_lib_init == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    return g_bridge_api.ds_lib_init();
}

int dsview_bridge_ds_lib_exit(void)
{
    if (g_bridge_api.ds_lib_exit == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    return g_bridge_api.ds_lib_exit();
}

void dsview_bridge_ds_set_firmware_resource_dir(const char *dir)
{
    if (g_bridge_api.ds_set_firmware_resource_dir != NULL) {
        g_bridge_api.ds_set_firmware_resource_dir(dir);
    }
}

int dsview_bridge_ds_get_device_list(struct ds_device_base_info **out_list, int *out_count)
{
    if (g_bridge_api.ds_get_device_list == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    return g_bridge_api.ds_get_device_list(out_list, out_count);
}

void dsview_bridge_free_device_list(struct ds_device_base_info *list)
{
    free(list);
}

int dsview_bridge_ds_active_device(ds_device_handle handle)
{
    if (g_bridge_api.ds_active_device == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    return g_bridge_api.ds_active_device(handle);
}

int dsview_bridge_ds_release_actived_device(void)
{
    if (g_bridge_api.ds_release_actived_device == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    return g_bridge_api.ds_release_actived_device();
}

int dsview_bridge_ds_get_last_error(void)
{
    if (g_bridge_api.ds_get_last_error == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    return g_bridge_api.ds_get_last_error();
}

int dsview_bridge_ds_get_actived_device_init_status(int *status)
{
    if (g_bridge_api.ds_get_actived_device_init_status == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    return g_bridge_api.ds_get_actived_device_init_status(status);
}

int dsview_bridge_ds_get_current_samplerate(unsigned long long *value)
{
    return dsview_bridge_get_uint64_config(SR_CONF_SAMPLERATE, value);
}

int dsview_bridge_ds_get_current_sample_limit(unsigned long long *value)
{
    return dsview_bridge_get_uint64_config(SR_CONF_LIMIT_SAMPLES, value);
}

int dsview_bridge_ds_get_total_channel_count(int *value)
{
    return dsview_bridge_get_int16_config(SR_CONF_TOTAL_CH_NUM, value);
}

int dsview_bridge_ds_get_valid_channel_count(int *value)
{
    return dsview_bridge_get_int16_config(SR_CONF_VLD_CH_NUM, value);
}

int dsview_bridge_ds_get_current_channel_mode(int *value)
{
    return dsview_bridge_get_int16_config(SR_CONF_CHANNEL_MODE, value);
}

int dsview_bridge_ds_get_hw_depth(unsigned long long *value)
{
    return dsview_bridge_get_uint64_config(SR_CONF_HW_DEPTH, value);
}

int dsview_bridge_ds_get_vth(double *value)
{
    return dsview_bridge_get_double_config(SR_CONF_VTH, value);
}

int dsview_bridge_ds_get_samplerates(struct dsview_samplerate_list *out_list)
{
    GVariant *data = NULL;
    GVariant *samplerates = NULL;
    gsize count = 0;
    const guint64 *values = NULL;
    int status;
    unsigned int i;

    if (out_list == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (g_bridge_api.ds_get_actived_device_config_list == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    memset(out_list, 0, sizeof(*out_list));
    status = g_bridge_api.ds_get_actived_device_config_list(NULL, SR_CONF_SAMPLERATE, &data);
    if (status != SR_OK) {
        return status;
    }
    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    samplerates = g_variant_lookup_value(data, "samplerates", G_VARIANT_TYPE("at"));
    if (samplerates == NULL) {
        g_variant_unref(data);
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    values = g_variant_get_fixed_array(samplerates, &count, sizeof(guint64));
    if (count > 64) {
        count = 64;
    }
    out_list->count = (unsigned int)count;
    for (i = 0; i < out_list->count; i++) {
        out_list->values[i] = values[i];
    }

    g_variant_unref(samplerates);
    g_variant_unref(data);
    return SR_OK;
}

int dsview_bridge_ds_get_channel_modes(struct dsview_channel_mode *out_modes, int max_modes, int *out_count)
{
    GVariant *data = NULL;
    struct sr_list_item *items = NULL;
    int index = 0;
    int status;

    if (out_count == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (g_bridge_api.ds_get_actived_device_config_list == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    *out_count = 0;
    status = g_bridge_api.ds_get_actived_device_config_list(NULL, SR_CONF_CHANNEL_MODE, &data);
    if (status != SR_OK) {
        return status;
    }
    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    items = (struct sr_list_item *)(uintptr_t)g_variant_get_uint64(data);
    while (items != NULL && items[index].id >= 0) {
        if (out_modes != NULL && index < max_modes) {
            out_modes[index].id = items[index].id;
            out_modes[index].max_enabled_channels = 0;
            memset(out_modes[index].name, 0, sizeof(out_modes[index].name));
            if (items[index].name != NULL) {
                strncpy(out_modes[index].name, items[index].name, sizeof(out_modes[index].name) - 1);
                out_modes[index].max_enabled_channels = dsview_bridge_mode_max_enabled_channels(items[index].name);
            }
        }
        index++;
    }

    g_variant_unref(data);
    *out_count = index;
    return SR_OK;
}

int dsview_bridge_ds_set_samplerate(unsigned long long value)
{
    GVariant *data;

    if (g_bridge_api.ds_set_actived_device_config == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    data = g_variant_new_uint64(value);
    return g_bridge_api.ds_set_actived_device_config(NULL, NULL, SR_CONF_SAMPLERATE, data);
}

int dsview_bridge_ds_set_sample_limit(unsigned long long value)
{
    GVariant *data;

    if (g_bridge_api.ds_set_actived_device_config == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    data = g_variant_new_uint64(value);
    return g_bridge_api.ds_set_actived_device_config(NULL, NULL, SR_CONF_LIMIT_SAMPLES, data);
}

int dsview_bridge_ds_enable_channel(int channel_index, int enable)
{
    if (g_bridge_api.ds_enable_device_channel_index == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    return g_bridge_api.ds_enable_device_channel_index(channel_index, enable ? TRUE : FALSE);
}

static void dsview_bridge_record_terminal_event(int terminal_event)
{
    g_acquisition_summary.terminal_event = terminal_event;
    if (terminal_event == DSVIEW_ACQ_TERMINAL_NORMAL_END) {
        g_acquisition_summary.saw_terminal_normal_end = 1;
    } else if (terminal_event == DSVIEW_ACQ_TERMINAL_END_BY_DETACHED) {
        g_acquisition_summary.saw_terminal_end_by_detached = 1;
    } else if (terminal_event == DSVIEW_ACQ_TERMINAL_END_BY_ERROR) {
        g_acquisition_summary.saw_terminal_end_by_error = 1;
    }
}

static void dsview_bridge_event_callback(int event)
{
    switch (event) {
    case DS_EV_COLLECT_TASK_START:
        g_acquisition_summary.saw_collect_task_start = 1;
        break;
    case DS_EV_DEVICE_RUNNING:
        g_acquisition_summary.saw_device_running = 1;
        break;
    case DS_EV_DEVICE_STOPPED:
        g_acquisition_summary.saw_device_stopped = 1;
        break;
    case DS_EV_COLLECT_TASK_END:
        dsview_bridge_record_terminal_event(DSVIEW_ACQ_TERMINAL_NORMAL_END);
        break;
    case DS_EV_COLLECT_TASK_END_BY_DETACHED:
        dsview_bridge_record_terminal_event(DSVIEW_ACQ_TERMINAL_END_BY_DETACHED);
        break;
    case DS_EV_COLLECT_TASK_END_BY_ERROR:
        dsview_bridge_record_terminal_event(DSVIEW_ACQ_TERMINAL_END_BY_ERROR);
        break;
    default:
        break;
    }
}

static void dsview_bridge_datafeed_callback(const struct sr_dev_inst *sdi, const struct sr_datafeed_packet *packet)
{
    int status;

    (void)sdi;

    if (packet == NULL) {
        return;
    }

    if (packet->type == SR_DF_LOGIC) {
        g_acquisition_summary.saw_logic_packet = 1;
    } else if (packet->type == SR_DF_END) {
        g_acquisition_summary.saw_end_packet = 1;
        g_acquisition_summary.end_packet_status = packet->status;
        if (packet->status == SR_PKT_OK) {
            g_acquisition_summary.saw_end_packet_ok = 1;
        } else {
            g_acquisition_summary.saw_data_error_packet = 1;
        }
    }

    status = dsview_bridge_record_packet(packet);
    if (status == DSVIEW_EXPORT_ERR_OVERFLOW) {
        g_recorded_stream.overflowed = 1;
    }
}

static int dsview_bridge_capture_is_collecting(void)
{
    if (g_bridge_api.ds_is_collecting == NULL) {
        return 0;
    }

    return g_bridge_api.ds_is_collecting();
}

static void dsview_bridge_refresh_collecting_flag(void)
{
    g_acquisition_summary.is_collecting = dsview_bridge_capture_is_collecting() ? 1 : 0;
}

static void dsview_bridge_refresh_last_error(void)
{
    if (g_bridge_api.ds_get_last_error == NULL) {
        g_acquisition_summary.last_error = DSVIEW_BRIDGE_ERR_NOT_LOADED;
        return;
    }

    g_acquisition_summary.last_error = g_bridge_api.ds_get_last_error();
}

static void dsview_bridge_clear_registered_callbacks(void)
{
    if (g_bridge_api.ds_set_event_callback != NULL) {
        g_bridge_api.ds_set_event_callback(NULL);
    }
    if (g_bridge_api.ds_set_datafeed_callback != NULL) {
        g_bridge_api.ds_set_datafeed_callback(NULL);
    }

    g_acquisition_callback_registration_active = 0;
    g_acquisition_summary.callback_registration_active = 0;
}

int dsview_bridge_ds_register_acquisition_callbacks(void)
{
    int status;

    if (g_bridge_api.ds_set_event_callback == NULL || g_bridge_api.ds_set_datafeed_callback == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }
    if (g_acquisition_callback_registration_active) {
        return SR_OK;
    }

    status = dsview_bridge_prepare_recording_capacity();
    if (status != SR_OK) {
        return status;
    }

    g_bridge_api.ds_set_event_callback((void *)(dslib_event_callback_t)dsview_bridge_event_callback);
    g_bridge_api.ds_set_datafeed_callback((void *)(ds_datafeed_callback_t)dsview_bridge_datafeed_callback);
    g_acquisition_callback_registration_active = 1;
    g_acquisition_summary.callback_registration_active = 1;
    return SR_OK;
}

int dsview_bridge_ds_clear_acquisition_callbacks(void)
{
    if (g_bridge_api.ds_set_event_callback == NULL || g_bridge_api.ds_set_datafeed_callback == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    dsview_bridge_clear_registered_callbacks();
    return SR_OK;
}

int dsview_bridge_ds_start_collect(void)
{
    int status;

    if (g_bridge_api.ds_start_collect == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    status = g_bridge_api.ds_start_collect();
    g_acquisition_summary.start_status = status;
    dsview_bridge_refresh_last_error();
    dsview_bridge_refresh_collecting_flag();
    return status;
}

int dsview_bridge_ds_stop_collect(void)
{
    int status;

    if (g_bridge_api.ds_stop_collect == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    status = g_bridge_api.ds_stop_collect();
    dsview_bridge_refresh_last_error();
    dsview_bridge_refresh_collecting_flag();
    return status;
}

int dsview_bridge_ds_is_collecting(int *value)
{
    if (value == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (g_bridge_api.ds_is_collecting == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    *value = dsview_bridge_capture_is_collecting() ? 1 : 0;
    g_acquisition_summary.is_collecting = *value;
    return SR_OK;
}

int dsview_bridge_ds_reset_acquisition_summary(void)
{
    memset(&g_acquisition_summary, 0, sizeof(g_acquisition_summary));
    g_acquisition_summary.terminal_event = DSVIEW_ACQ_TERMINAL_NONE;
    g_acquisition_summary.end_packet_status = -1;
    g_acquisition_summary.callback_registration_active = g_acquisition_callback_registration_active ? 1 : 0;
    dsview_bridge_reset_recorded_stream();
    dsview_bridge_refresh_last_error();
    dsview_bridge_refresh_collecting_flag();
    return SR_OK;
}

int dsview_bridge_ds_get_acquisition_summary(struct dsview_bridge_acquisition_summary *out_summary)
{
    if (out_summary == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    dsview_bridge_refresh_last_error();
    dsview_bridge_refresh_collecting_flag();
    *out_summary = g_acquisition_summary;
    return SR_OK;
}

static int dsview_bridge_build_vcd_device(const struct dsview_vcd_export_request *request, struct sr_dev_inst **out_sdi)
{
    struct sr_dev_inst *sdi;
    struct sr_channel *channel;
    GSList *node = NULL;
    size_t index;
    char name[16];

    if (request == NULL || out_sdi == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (request->enabled_channel_count == 0) {
        return DSVIEW_EXPORT_ERR_NO_ENABLED_CHANNELS;
    }

    sdi = calloc(1, sizeof(*sdi));
    if (sdi == NULL) {
        return SR_ERR_MALLOC;
    }
    sdi->mode = LOGIC;
    sdi->status = SR_ST_ACTIVE;

    for (index = 0; index < request->enabled_channel_count; index++) {
        channel = calloc(1, sizeof(*channel));
        if (channel == NULL) {
            dsview_bridge_free_vcd_device(sdi);
            return SR_ERR_MALLOC;
        }

        channel->index = request->enabled_channels[index];
        channel->type = SR_CHANNEL_LOGIC;
        channel->enabled = TRUE;
        snprintf(name, sizeof(name), "D%u", (unsigned int)request->enabled_channels[index]);
        channel->name = g_strdup(name);
        if (channel->name == NULL) {
            free(channel);
            dsview_bridge_free_vcd_device(sdi);
            return SR_ERR_MALLOC;
        }

        node = g_slist_append(sdi->channels, channel);
        if (node == NULL) {
            g_free(channel->name);
            free(channel);
            dsview_bridge_free_vcd_device(sdi);
            return SR_ERR_MALLOC;
        }
        sdi->channels = node;
    }

    *out_sdi = sdi;
    return SR_OK;
}

static void dsview_bridge_free_vcd_device(struct sr_dev_inst *sdi)
{
    GSList *item;

    if (sdi == NULL) {
        return;
    }

    for (item = sdi->channels; item != NULL; item = item->next) {
        struct sr_channel *channel = item->data;
        if (channel != NULL) {
            g_free(channel->name);
            free(channel);
        }
    }
    g_slist_free(sdi->channels);
    free(sdi);
}

static int dsview_bridge_append_output_chunk(GString **assembled_output, GString *chunk)
{
    if (chunk == NULL) {
        return SR_OK;
    }

    if (*assembled_output == NULL) {
        *assembled_output = g_string_sized_new(chunk->len + 256);
        if (*assembled_output == NULL) {
            g_string_free(chunk, TRUE);
            return SR_ERR_MALLOC;
        }
    }

    g_string_append_len(*assembled_output, chunk->str, chunk->len);
    g_string_free(chunk, TRUE);
    return SR_OK;
}

static int dsview_bridge_emit_packet(const struct sr_output *output, const struct dsview_retained_packet *packet, GString **assembled_output)
{
    struct sr_datafeed_packet replay_packet;
    struct sr_datafeed_meta meta;
    struct sr_datafeed_logic logic;
    struct sr_config config;
    GSList config_node;
    GString *chunk = NULL;
    int status;

    memset(&replay_packet, 0, sizeof(replay_packet));
    replay_packet.status = packet->status;
    replay_packet.bExportOriginalData = 0;

    switch (packet->type) {
    case DSVIEW_EXPORT_PACKET_META:
        memset(&meta, 0, sizeof(meta));
        memset(&config, 0, sizeof(config));
        memset(&config_node, 0, sizeof(config_node));
        config.key = SR_CONF_SAMPLERATE;
        config.data = g_variant_ref_sink(g_variant_new_uint64(packet->samplerate_hz));
        config_node.data = &config;
        meta.config = &config_node;
        replay_packet.type = SR_DF_META;
        replay_packet.payload = &meta;
        status = g_bridge_api.sr_output_send(output, &replay_packet, &chunk);
        g_variant_unref(config.data);
        break;
    case DSVIEW_EXPORT_PACKET_LOGIC:
        memset(&logic, 0, sizeof(logic));
        logic.length = packet->length;
        logic.unitsize = packet->unitsize;
        logic.data_error = packet->data_error;
        logic.error_pattern = packet->error_pattern;
        logic.data = packet->data;
        replay_packet.type = SR_DF_LOGIC;
        replay_packet.payload = &logic;
        status = g_bridge_api.sr_output_send(output, &replay_packet, &chunk);
        break;
    case DSVIEW_EXPORT_PACKET_END:
        replay_packet.type = SR_DF_END;
        replay_packet.payload = NULL;
        status = g_bridge_api.sr_output_send(output, &replay_packet, &chunk);
        break;
    default:
        return SR_OK;
    }

    if (status != SR_OK) {
        if (chunk != NULL) {
            g_string_free(chunk, TRUE);
        }
        return status;
    }

    return dsview_bridge_append_output_chunk(assembled_output, chunk);
}

static int dsview_bridge_export_stream(const struct dsview_vcd_export_request *request, const struct dsview_recorded_stream *stream, struct dsview_export_buffer *out_buffer)
{
    struct sr_dev_inst *sdi = NULL;
    const struct sr_output_module *module;
    const struct sr_output *output = NULL;
    GString *assembled_output = NULL;
    size_t index;
    int status = SR_OK;
    int saw_meta = 0;
    int saw_end = 0;
    unsigned long long replay_samplerate_hz;

    if (request == NULL || out_buffer == NULL || stream == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (g_bridge_api.sr_output_find == NULL || g_bridge_api.sr_output_new == NULL
        || g_bridge_api.sr_output_send == NULL || g_bridge_api.sr_output_free == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }
    if (stream->overflowed) {
        return DSVIEW_EXPORT_ERR_OVERFLOW;
    }
    if (!stream->saw_logic_packet || !stream->saw_end_packet || stream->packet_count == 0) {
        return DSVIEW_EXPORT_ERR_NO_STREAM;
    }
    if (stream->end_packet_status != SR_PKT_OK) {
        return DSVIEW_EXPORT_ERR_BAD_END_STATUS;
    }
    if (request->samplerate_hz == 0) {
        return DSVIEW_EXPORT_ERR_MISSING_SAMPLERATE;
    }

    replay_samplerate_hz = stream->samplerate_hz != 0 ? stream->samplerate_hz : request->samplerate_hz;

    module = g_bridge_api.sr_output_find("vcd");
    if (module == NULL) {
        return DSVIEW_EXPORT_ERR_OUTPUT_MODULE;
    }

    status = dsview_bridge_build_vcd_device(request, &sdi);
    if (status != SR_OK) {
        return status;
    }

    output = g_bridge_api.sr_output_new(module, NULL, sdi);
    if (output == NULL) {
        status = DSVIEW_EXPORT_ERR_OUTPUT_MODULE;
        goto cleanup;
    }

    for (index = 0; index < stream->packet_count; index++) {
        struct dsview_retained_packet replay_packet = stream->packets[index];

        if (replay_packet.type == DSVIEW_EXPORT_PACKET_META) {
            if (replay_packet.samplerate_hz == 0) {
                replay_packet.samplerate_hz = replay_samplerate_hz;
            }
            saw_meta = 1;
        } else if (replay_packet.type == DSVIEW_EXPORT_PACKET_LOGIC && !saw_meta) {
            struct dsview_retained_packet meta_packet;
            memset(&meta_packet, 0, sizeof(meta_packet));
            meta_packet.type = DSVIEW_EXPORT_PACKET_META;
            meta_packet.status = SR_PKT_OK;
            meta_packet.samplerate_hz = replay_samplerate_hz;
            status = dsview_bridge_emit_packet(output, &meta_packet, &assembled_output);
            if (status != SR_OK) {
                goto cleanup;
            }
            saw_meta = 1;
        } else if (replay_packet.type == DSVIEW_EXPORT_PACKET_END) {
            saw_end = 1;
        }

        status = dsview_bridge_emit_packet(output, &replay_packet, &assembled_output);
        if (status != SR_OK) {
            goto cleanup;
        }
    }

    if (!saw_meta) {
        struct dsview_retained_packet meta_packet;
        memset(&meta_packet, 0, sizeof(meta_packet));
        meta_packet.type = DSVIEW_EXPORT_PACKET_META;
        meta_packet.status = SR_PKT_OK;
        meta_packet.samplerate_hz = replay_samplerate_hz;
        status = dsview_bridge_emit_packet(output, &meta_packet, &assembled_output);
        if (status != SR_OK) {
            goto cleanup;
        }
    }

    if (!saw_end) {
        status = DSVIEW_EXPORT_ERR_NO_STREAM;
        goto cleanup;
    }

    if (assembled_output == NULL) {
        status = DSVIEW_EXPORT_ERR_NO_STREAM;
        goto cleanup;
    }

    out_buffer->data = (uint8_t *)g_string_free(assembled_output, FALSE);
    assembled_output = NULL;
    out_buffer->len = out_buffer->data != NULL ? strlen((char *)out_buffer->data) : 0;
    out_buffer->sample_count = stream->sample_count;
    out_buffer->packet_count = stream->packet_count;
    status = SR_OK;

cleanup:
    if (assembled_output != NULL) {
        g_string_free(assembled_output, TRUE);
    }
    if (output != NULL) {
        g_bridge_api.sr_output_free(output);
    }
    dsview_bridge_free_vcd_device(sdi);
    return status;
}

int dsview_bridge_ds_export_recorded_vcd(
    const struct dsview_vcd_export_request *request,
    struct dsview_export_buffer *out_buffer)
{
    if (out_buffer == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    memset(out_buffer, 0, sizeof(*out_buffer));
    return dsview_bridge_export_stream(request, &g_recorded_stream, out_buffer);
}

int dsview_bridge_render_vcd_from_samples(
    const struct dsview_vcd_export_request *request,
    const uint8_t *sample_bytes,
    size_t sample_bytes_len,
    uint16_t unitsize,
    struct dsview_export_buffer *out_buffer)
{
    struct dsview_recorded_stream stream;
    struct dsview_retained_packet packets[3];

    if (request == NULL || sample_bytes == NULL || sample_bytes_len == 0 || unitsize == 0 || out_buffer == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if ((sample_bytes_len % unitsize) != 0) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    memset(&stream, 0, sizeof(stream));
    memset(&packets, 0, sizeof(packets));
    memset(out_buffer, 0, sizeof(*out_buffer));

    packets[0].type = DSVIEW_EXPORT_PACKET_META;
    packets[0].status = SR_PKT_OK;
    packets[0].samplerate_hz = request->samplerate_hz;

    packets[1].type = DSVIEW_EXPORT_PACKET_LOGIC;
    packets[1].status = SR_PKT_OK;
    packets[1].length = sample_bytes_len;
    packets[1].unitsize = unitsize;
    packets[1].data = malloc(sample_bytes_len);
    if (packets[1].data == NULL) {
        return SR_ERR_MALLOC;
    }
    memcpy(packets[1].data, sample_bytes, sample_bytes_len);

    packets[2].type = DSVIEW_EXPORT_PACKET_END;
    packets[2].status = SR_PKT_OK;

    stream.packets = packets;
    stream.packet_count = 3;
    stream.saw_logic_packet = 1;
    stream.saw_end_packet = 1;
    stream.end_packet_status = SR_PKT_OK;
    stream.samplerate_hz = request->samplerate_hz;
    stream.has_samplerate = 1;
    stream.sample_count = sample_bytes_len / unitsize;

    int status = dsview_bridge_export_stream(request, &stream, out_buffer);
    free(packets[1].data);
    return status;
}

int dsview_bridge_render_vcd_from_logic_packets(
    const struct dsview_vcd_export_request *request,
    const uint8_t *sample_bytes,
    size_t sample_bytes_len,
    const size_t *logic_packet_lengths,
    size_t logic_packet_count,
    uint16_t unitsize,
    struct dsview_export_buffer *out_buffer)
{
    struct dsview_recorded_stream stream;
    struct dsview_retained_packet *packets = NULL;
    size_t packet_index;
    size_t offset = 0;
    int status;

    if (request == NULL || sample_bytes == NULL || sample_bytes_len == 0 || logic_packet_lengths == NULL
        || logic_packet_count == 0 || unitsize == 0 || out_buffer == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if ((sample_bytes_len % unitsize) != 0) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    memset(&stream, 0, sizeof(stream));
    memset(out_buffer, 0, sizeof(*out_buffer));

    packets = calloc(logic_packet_count + 2, sizeof(*packets));
    if (packets == NULL) {
        return SR_ERR_MALLOC;
    }

    packets[0].type = DSVIEW_EXPORT_PACKET_META;
    packets[0].status = SR_PKT_OK;
    packets[0].samplerate_hz = request->samplerate_hz;

    for (packet_index = 0; packet_index < logic_packet_count; packet_index++) {
        size_t packet_length = logic_packet_lengths[packet_index];
        struct dsview_retained_packet *logic_packet = &packets[packet_index + 1];

        if (packet_length == 0 || (packet_length % unitsize) != 0 || offset > sample_bytes_len
            || packet_length > sample_bytes_len - offset) {
            status = DSVIEW_BRIDGE_ERR_ARG;
            goto cleanup;
        }

        logic_packet->type = DSVIEW_EXPORT_PACKET_LOGIC;
        logic_packet->status = SR_PKT_OK;
        logic_packet->length = packet_length;
        logic_packet->unitsize = unitsize;
        logic_packet->data = malloc(packet_length);
        if (logic_packet->data == NULL) {
            status = SR_ERR_MALLOC;
            goto cleanup;
        }
        memcpy(logic_packet->data, sample_bytes + offset, packet_length);
        offset += packet_length;
    }

    if (offset != sample_bytes_len) {
        status = DSVIEW_BRIDGE_ERR_ARG;
        goto cleanup;
    }

    packets[logic_packet_count + 1].type = DSVIEW_EXPORT_PACKET_END;
    packets[logic_packet_count + 1].status = SR_PKT_OK;

    stream.packets = packets;
    stream.packet_count = logic_packet_count + 2;
    stream.saw_logic_packet = 1;
    stream.saw_end_packet = 1;
    stream.end_packet_status = SR_PKT_OK;
    stream.samplerate_hz = request->samplerate_hz;
    stream.has_samplerate = 1;
    stream.sample_count = sample_bytes_len / unitsize;

    status = dsview_bridge_export_stream(request, &stream, out_buffer);

cleanup:
    if (packets != NULL) {
        for (packet_index = 0; packet_index < logic_packet_count; packet_index++) {
            free(packets[packet_index + 1].data);
        }
        free(packets);
    }
    return status;
}

void dsview_bridge_free_export_buffer(struct dsview_export_buffer *buffer)
{
    if (buffer == NULL) {
        return;
    }

    if (buffer->data != NULL) {
        g_free(buffer->data);
    }
    memset(buffer, 0, sizeof(*buffer));
}
