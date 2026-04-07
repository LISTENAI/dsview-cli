#include "wrapper.h"

#include <dlfcn.h>
#include <glib.h>
#include <stdlib.h>
#include <string.h>

typedef int (*ds_lib_init_fn)(void);
typedef int (*ds_lib_exit_fn)(void);
typedef void (*ds_set_firmware_resource_dir_fn)(const char *dir);
typedef int (*ds_get_device_list_fn)(struct ds_device_base_info **out_list, int *out_count);
typedef int (*ds_active_device_fn)(ds_device_handle handle);
typedef int (*ds_release_actived_device_fn)(void);
typedef int (*ds_get_last_error_fn)(void);
typedef int (*ds_get_actived_device_init_status_fn)(int *status);
typedef int (*ds_get_actived_device_config_fn)(const void *ch, const void *cg, int key, GVariant **data);
typedef int (*ds_get_actived_device_config_list_fn)(const void *cg, int key, GVariant **data);
typedef int (*ds_set_actived_device_config_fn)(const void *ch, const void *cg, int key, GVariant *data);
typedef int (*ds_set_event_callback_fn)(void *cb);
typedef int (*ds_set_datafeed_callback_fn)(void *cb);
typedef int (*ds_start_collect_fn)(void);
typedef int (*ds_stop_collect_fn)(void);
typedef int (*ds_is_collecting_fn)(void);
typedef int (*ds_enable_device_channel_index_fn)(int channel_index, gboolean enable);

typedef void (*dslib_event_callback_t)(int event);
typedef void (*ds_datafeed_callback_t)(const void *sdi, const struct sr_datafeed_packet *packet);

struct sr_datafeed_packet {
    int type;
    int status;
    const void *payload;
};

enum {
    SR_OK = 0,
    SR_PKT_OK = 0,
    SR_DF_END = 10001,
    SR_DF_LOGIC = 10004,
    DS_EV_COLLECT_TASK_START = 101,
    DS_EV_COLLECT_TASK_END = 102,
    DS_EV_DEVICE_RUNNING = 103,
    DS_EV_DEVICE_STOPPED = 104,
    DS_EV_COLLECT_TASK_END_BY_DETACHED = 105,
    DS_EV_COLLECT_TASK_END_BY_ERROR = 106,
    SR_CONF_SAMPLERATE = 30000,
    SR_CONF_VLD_CH_NUM = 30027,
    SR_CONF_TOTAL_CH_NUM = 30026,
    SR_CONF_CHANNEL_MODE = 30067,
    SR_CONF_VTH = 30072,
    SR_CONF_HW_DEPTH = 30075,
    SR_CONF_LIMIT_SAMPLES = 50001,
};

struct sr_list_item {
    int id;
    const char *name;
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
    char last_error[512];
};

static struct dsview_bridge_api g_bridge_api;
static struct dsview_bridge_acquisition_summary g_acquisition_summary;
static int g_acquisition_callback_registration_active = 0;

static void dsview_bridge_clear_registered_callbacks(void);

static void dsview_bridge_set_error_from_text(const char *message)
{
    if (message == NULL) {
        g_bridge_api.last_error[0] = '\0';
        return;
    }

    strncpy(g_bridge_api.last_error, message, sizeof(g_bridge_api.last_error) - 1);
    g_bridge_api.last_error[sizeof(g_bridge_api.last_error) - 1] = '\0';
}

static void dsview_bridge_set_error_from_dlerror(void)
{
    const char *error = dlerror();
    dsview_bridge_set_error_from_text(error != NULL ? error : "unknown dynamic loader error");
}

static void *dsview_bridge_load_symbol(const char *name, int *status_out)
{
    void *symbol = NULL;

    dlerror();
    symbol = dlsym(g_bridge_api.library_handle, name);
    if (symbol == NULL) {
        dsview_bridge_set_error_from_dlerror();
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

int dsview_bridge_load_library(const char *path)
{
    int status = DSVIEW_BRIDGE_OK;

    if (path == NULL || path[0] == '\0') {
        dsview_bridge_set_error_from_text("library path must not be empty");
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    dsview_bridge_unload_library();

    dlerror();
    g_bridge_api.library_handle = dlopen(path, RTLD_NOW | RTLD_LOCAL);
    if (g_bridge_api.library_handle == NULL) {
        dsview_bridge_set_error_from_dlerror();
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

    dsview_bridge_set_error_from_text(NULL);
    return DSVIEW_BRIDGE_OK;
}

void dsview_bridge_unload_library(void)
{
    dsview_bridge_clear_registered_callbacks();

    if (g_bridge_api.library_handle != NULL) {
        dlclose(g_bridge_api.library_handle);
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

static void dsview_bridge_datafeed_callback(const void *sdi, const struct sr_datafeed_packet *packet)
{
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
    if (g_bridge_api.ds_set_event_callback == NULL || g_bridge_api.ds_set_datafeed_callback == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }
    if (g_acquisition_callback_registration_active) {
        return SR_OK;
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

