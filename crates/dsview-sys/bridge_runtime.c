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

#define DSVIEW_BRIDGE_CHANNEL_TRACK_CAPACITY 64

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
typedef int (*srd_init_fn)(const char *path);
typedef int (*srd_exit_fn)(void);
typedef const GSList *(*srd_decoder_list_fn)(void);
typedef struct srd_decoder *(*srd_decoder_get_by_id_fn)(const char *id);
typedef int (*srd_decoder_load_all_fn)(void);
typedef GSList *(*srd_searchpaths_get_fn)(void);
typedef const char *(*srd_strerror_fn)(int error_code);
typedef const char *(*srd_strerror_name_fn)(int error_code);
typedef int (*srd_session_new_fn)(struct srd_session **sess);
typedef int (*srd_session_metadata_set_fn)(struct srd_session *sess, int key, GVariant *data);
typedef int (*srd_session_start_fn)(struct srd_session *sess, char **error);
typedef int (*srd_session_send_fn)(
    struct srd_session *sess,
    uint64_t abs_start_samplenum,
    uint64_t abs_end_samplenum,
    const uint8_t **inbuf,
    const uint8_t *inbuf_const,
    uint64_t inbuflen,
    char **error);
typedef int (*srd_session_end_fn)(struct srd_session *sess, char **error);
typedef int (*srd_session_destroy_fn)(struct srd_session *sess);
typedef int (*srd_pd_output_callback_add_fn)(
    struct srd_session *sess,
    int output_type,
    srd_pd_output_callback cb,
    void *cb_data);
typedef struct srd_decoder_inst *(*srd_inst_new_fn)(
    struct srd_session *sess,
    const char *decoder_id,
    GHashTable *options);
typedef int (*srd_inst_channel_set_all_fn)(struct srd_decoder_inst *di, GHashTable *channels);
typedef int (*srd_inst_stack_fn)(
    struct srd_session *sess,
    struct srd_decoder_inst *di_bottom,
    struct srd_decoder_inst *di_top);

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

struct dsview_decode_runtime_api {
    void *library_handle;
    srd_init_fn srd_init;
    srd_exit_fn srd_exit;
    srd_decoder_list_fn srd_decoder_list;
    srd_decoder_get_by_id_fn srd_decoder_get_by_id;
    srd_decoder_load_all_fn srd_decoder_load_all;
    srd_searchpaths_get_fn srd_searchpaths_get;
    srd_strerror_fn srd_strerror;
    srd_strerror_name_fn srd_strerror_name;
    srd_session_new_fn srd_session_new;
    srd_session_metadata_set_fn srd_session_metadata_set;
    srd_session_start_fn srd_session_start;
    srd_session_send_fn srd_session_send;
    srd_session_end_fn srd_session_end;
    srd_session_destroy_fn srd_session_destroy;
    srd_pd_output_callback_add_fn srd_pd_output_callback_add;
    srd_inst_new_fn srd_inst_new;
    srd_inst_channel_set_all_fn srd_inst_channel_set_all;
    srd_inst_stack_fn srd_inst_stack;
    char last_loader_error[512];
    char last_error[512];
    char last_error_name[128];
    int last_error_code;
    int initialized;
};

struct dsview_decode_execution_session {
    struct srd_session *session;
    struct srd_decoder_inst *root;
    unsigned int input_channel_count;
    int started;
    int ended;
    GMutex annotation_lock;
    struct dsview_decode_captured_annotation *captured_annotations;
    size_t captured_annotation_count;
    size_t captured_annotation_capacity;
};

struct dsview_retained_packet {
    int type;
    int status;
    int format;
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
    uint16_t enabled_channel_count;
};

static struct dsview_bridge_api g_bridge_api;
static struct dsview_decode_runtime_api g_decode_runtime_api;
static struct dsview_bridge_acquisition_summary g_acquisition_summary;
static int g_acquisition_callback_registration_active = 0;
static struct dsview_recorded_stream g_recorded_stream;
static uint8_t g_enabled_channel_state[DSVIEW_BRIDGE_CHANNEL_TRACK_CAPACITY];

static void dsview_bridge_clear_registered_callbacks(void);
static void dsview_bridge_reset_recorded_stream(void);
static int dsview_bridge_prepare_recording_capacity(void);
static int dsview_bridge_record_packet(const struct sr_datafeed_packet *packet);
static int dsview_bridge_export_stream(const struct dsview_vcd_export_request *request, const struct dsview_recorded_stream *stream, struct dsview_export_buffer *out_buffer);
static int dsview_bridge_build_vcd_device(const struct dsview_vcd_export_request *request, struct sr_dev_inst **out_sdi);
static void dsview_bridge_free_vcd_device(struct sr_dev_inst *sdi);
static int dsview_bridge_emit_packet(const struct sr_output *output, const struct dsview_retained_packet *packet, GString **assembled_output);
static int dsview_bridge_emit_cross_logic_packet(
    const struct sr_output *output,
    const struct dsview_retained_packet *packet,
    uint16_t enabled_channel_count,
    GString **assembled_output);
static uint16_t dsview_bridge_expected_logic_unitsize(void);
static int dsview_bridge_get_optional_int16_config(int key, int *has_value, int *value);
static int dsview_bridge_get_optional_double_config(int key, int *has_value, double *value);
static int dsview_bridge_set_int16_config(int key, int value);
static int dsview_bridge_copy_option_values(int key, struct dsview_option_value *out_values, int max_values, unsigned short *out_count);
static int dsview_bridge_copy_channel_modes_for_current_operation(struct dsview_channel_mode *out_modes, int max_modes, unsigned short *out_count);
static int dsview_bridge_restore_device_modes(int has_operation_mode, int operation_mode, int has_channel_mode, int channel_mode);
static void dsview_bridge_copy_string(char *dst, size_t dst_len, const char *src);
static void dsview_decode_clear_error_from_gchar(char *message);
static int dsview_decode_invalid_shape(const char *detail);
static int dsview_decode_invalid_session(const char *detail);
static GHashTable *dsview_decode_build_option_table(
    const struct dsview_decode_option_entry *options,
    size_t option_count);
static GHashTable *dsview_decode_build_channel_table(
    const struct dsview_decode_channel_binding *bindings,
    size_t binding_count,
    unsigned int *out_input_channel_count);
static struct srd_decoder_inst *dsview_decode_create_instance(
    struct dsview_decode_execution_session *session,
    const struct dsview_decode_instance_spec *spec,
    int allow_channel_bindings,
    unsigned int *out_input_channel_count);
static int dsview_decode_build_split_chunk(
    const struct dsview_decode_execution_session *session,
    const struct dsview_decode_logic_chunk *chunk,
    const uint8_t ***out_inbuf,
    uint8_t **out_inbuf_const,
    uint8_t ***out_owned_buffers,
    uint64_t *out_sample_count);
static int dsview_decode_build_cross_chunk(
    const struct dsview_decode_execution_session *session,
    const struct dsview_decode_logic_chunk *chunk,
    const uint8_t ***out_inbuf,
    uint8_t **out_inbuf_const,
    uint8_t ***out_owned_buffers,
    uint64_t *out_sample_count);
static void dsview_decode_free_chunk_buffers(
    const struct dsview_decode_execution_session *session,
    const uint8_t **inbuf,
    uint8_t *inbuf_const,
    uint8_t **owned_buffers);
static void dsview_decode_free_captured_annotations_internal(
    struct dsview_decode_captured_annotation *annotations,
    size_t count);
static int dsview_decode_reserve_captured_annotations(
    struct dsview_decode_execution_session *session,
    size_t target_count);
static void dsview_decode_annotation_callback(
    struct srd_proto_data *pdata,
    void *cb_data);

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

static void *dsview_bridge_dlopen_global(const char *path)
{
    /*
     * Win32 does not distinguish local/global symbol visibility at load time
     * the way dlopen() does on ELF platforms, so both helpers resolve through
     * LoadLibraryA().
     */
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

static void *dsview_bridge_dlopen_global(const char *path)
{
    return dlopen(path, RTLD_NOW | RTLD_GLOBAL);
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

static void dsview_decode_copy_text(char *dst, size_t dst_len, const char *src)
{
    if (dst == NULL || dst_len == 0) {
        return;
    }

    if (src == NULL) {
        dst[0] = '\0';
        return;
    }

    strncpy(dst, src, dst_len - 1);
    dst[dst_len - 1] = '\0';
}

static char *dsview_decode_strdup(const char *value)
{
    size_t len;
    char *copy;

    if (value == NULL) {
        return NULL;
    }

    len = strlen(value) + 1;
    copy = (char *)malloc(len);
    if (copy == NULL) {
        return NULL;
    }

    memcpy(copy, value, len);
    return copy;
}

static void dsview_decode_clear_error_state(void)
{
    g_decode_runtime_api.last_error[0] = '\0';
    g_decode_runtime_api.last_error_name[0] = '\0';
    g_decode_runtime_api.last_error_code = DSVIEW_DECODE_OK;
}

static void dsview_decode_set_loader_error_text(const char *message)
{
    dsview_decode_copy_text(
        g_decode_runtime_api.last_loader_error,
        sizeof(g_decode_runtime_api.last_loader_error),
        message);
}

#if defined(_WIN32)
static void dsview_decode_set_loader_error_from_loader(void)
{
    DWORD error = GetLastError();
    char buffer[128];

    if (error == ERROR_SUCCESS) {
        dsview_decode_set_loader_error_text("unknown dynamic loader error");
        return;
    }

    snprintf(buffer, sizeof(buffer), "dynamic loader error code %lu", (unsigned long)error);
    dsview_decode_set_loader_error_text(buffer);
}
#else
static void dsview_decode_set_loader_error_from_loader(void)
{
    const char *error = dlerror();
    dsview_decode_set_loader_error_text(error != NULL ? error : "unknown dynamic loader error");
}
#endif

static void dsview_decode_set_error_detail(int code, const char *name, const char *detail)
{
    g_decode_runtime_api.last_error_code = code;
    dsview_decode_copy_text(
        g_decode_runtime_api.last_error_name,
        sizeof(g_decode_runtime_api.last_error_name),
        name);
    dsview_decode_copy_text(
        g_decode_runtime_api.last_error,
        sizeof(g_decode_runtime_api.last_error),
        detail);
}

static int dsview_decode_map_upstream_status(int status, int during_decoder_load)
{
    if (status == SRD_ERR_DECODERS_DIR) {
        return DSVIEW_DECODE_ERR_DECODER_DIR;
    }
    if (status == SRD_ERR_PYTHON) {
        return during_decoder_load ? DSVIEW_DECODE_ERR_DECODER_LOAD : DSVIEW_DECODE_ERR_PYTHON;
    }
    if (status == SRD_ERR_MALLOC) {
        return DSVIEW_DECODE_ERR_MALLOC;
    }
    if (during_decoder_load) {
        return DSVIEW_DECODE_ERR_DECODER_LOAD;
    }
    return DSVIEW_DECODE_ERR_UPSTREAM;
}

static void dsview_decode_capture_upstream_error(int status, int during_decoder_load)
{
    const char *name = NULL;
    const char *detail = NULL;
    int mapped_code = dsview_decode_map_upstream_status(status, during_decoder_load);

    if (g_decode_runtime_api.srd_strerror_name != NULL) {
        name = g_decode_runtime_api.srd_strerror_name(status);
    }
    if (g_decode_runtime_api.srd_strerror != NULL) {
        detail = g_decode_runtime_api.srd_strerror(status);
    }

    dsview_decode_set_error_detail(
        mapped_code,
        name != NULL ? name : "unknown error code",
        detail != NULL ? detail : "unknown error");
}

static void dsview_decode_clear_error_from_gchar(char *message)
{
    if (message == NULL) {
        return;
    }

    dsview_decode_set_error_detail(DSVIEW_DECODE_ERR_PYTHON, "python error", message);
    g_free(message);
}

static int dsview_decode_invalid_shape(const char *detail)
{
    dsview_decode_set_error_detail(
        DSVIEW_DECODE_ERR_INPUT_SHAPE,
        "invalid input shape",
        detail != NULL ? detail : "logic chunk shape is invalid");
    return DSVIEW_DECODE_ERR_INPUT_SHAPE;
}

static int dsview_decode_invalid_session(const char *detail)
{
    dsview_decode_set_error_detail(
        DSVIEW_DECODE_ERR_SESSION,
        "invalid decode session state",
        detail != NULL ? detail : "decode session state is invalid");
    return DSVIEW_DECODE_ERR_SESSION;
}

static int dsview_decode_not_loaded(void)
{
    dsview_decode_set_error_detail(
        DSVIEW_DECODE_ERR_NOT_LOADED,
        "decode runtime not loaded",
        "load the decode runtime library before calling this operation");
    return DSVIEW_DECODE_ERR_NOT_LOADED;
}

static int dsview_decode_not_initialized(void)
{
    dsview_decode_set_error_detail(
        DSVIEW_DECODE_ERR_NOT_LOADED,
        "decode runtime not initialized",
        "initialize the decode runtime before listing or inspecting decoders");
    return DSVIEW_DECODE_ERR_NOT_LOADED;
}

static void *dsview_decode_load_symbol(const char *name, int *status_out)
{
    void *symbol = NULL;

    dsview_bridge_clear_loader_error();
    symbol = dsview_bridge_dlsym(g_decode_runtime_api.library_handle, name);
    if (symbol == NULL) {
        dsview_decode_set_loader_error_from_loader();
        if (status_out != NULL) {
            *status_out = DSVIEW_BRIDGE_ERR_DLSYM;
        }
    }

    return symbol;
}

static void dsview_decode_free_string_array(char **items, size_t count)
{
    size_t index;

    if (items == NULL) {
        return;
    }

    for (index = 0; index < count; index++) {
        free(items[index]);
    }
    free(items);
}

static int dsview_decode_copy_string_array(const GSList *source, char ***out_items, size_t *out_count)
{
    const GSList *node;
    char **items = NULL;
    size_t count = 0;
    size_t index = 0;

    if (out_items == NULL || out_count == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }

    *out_items = NULL;
    *out_count = 0;

    count = (size_t)g_slist_length((GSList *)source);
    if (count == 0) {
        return DSVIEW_DECODE_OK;
    }

    items = (char **)calloc(count, sizeof(char *));
    if (items == NULL) {
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    for (node = source; node != NULL; node = node->next, index++) {
        items[index] = dsview_decode_strdup((const char *)node->data);
        if (items[index] == NULL) {
            dsview_decode_free_string_array(items, count);
            return DSVIEW_DECODE_ERR_MALLOC;
        }
    }

    *out_items = items;
    *out_count = count;
    return DSVIEW_DECODE_OK;
}

static void dsview_decode_free_channels(struct dsview_decode_channel *channels, size_t count)
{
    size_t index;

    if (channels == NULL) {
        return;
    }

    for (index = 0; index < count; index++) {
        free(channels[index].id);
        free(channels[index].name);
        free(channels[index].desc);
        free(channels[index].idn);
    }
    free(channels);
}

static int dsview_decode_copy_channels(
    const GSList *source,
    struct dsview_decode_channel **out_channels,
    size_t *out_count)
{
    const GSList *node;
    struct dsview_decode_channel *channels = NULL;
    size_t count = 0;
    size_t index = 0;
    const struct srd_channel *source_channel;

    if (out_channels == NULL || out_count == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }

    *out_channels = NULL;
    *out_count = 0;
    count = (size_t)g_slist_length((GSList *)source);
    if (count == 0) {
        return DSVIEW_DECODE_OK;
    }

    channels = (struct dsview_decode_channel *)calloc(count, sizeof(*channels));
    if (channels == NULL) {
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    for (node = source; node != NULL; node = node->next, index++) {
        source_channel = (const struct srd_channel *)node->data;
        channels[index].id = dsview_decode_strdup(source_channel->id);
        channels[index].name = dsview_decode_strdup(source_channel->name);
        channels[index].desc = dsview_decode_strdup(source_channel->desc);
        channels[index].idn = dsview_decode_strdup(source_channel->idn);
        channels[index].order = source_channel->order;
        channels[index].type = source_channel->type;

        if (source_channel->id != NULL && channels[index].id == NULL) {
            dsview_decode_free_channels(channels, count);
            return DSVIEW_DECODE_ERR_MALLOC;
        }
        if (source_channel->name != NULL && channels[index].name == NULL) {
            dsview_decode_free_channels(channels, count);
            return DSVIEW_DECODE_ERR_MALLOC;
        }
        if (source_channel->desc != NULL && channels[index].desc == NULL) {
            dsview_decode_free_channels(channels, count);
            return DSVIEW_DECODE_ERR_MALLOC;
        }
        if (source_channel->idn != NULL && channels[index].idn == NULL) {
            dsview_decode_free_channels(channels, count);
            return DSVIEW_DECODE_ERR_MALLOC;
        }
    }

    *out_channels = channels;
    *out_count = count;
    return DSVIEW_DECODE_OK;
}

static void dsview_decode_free_options(struct dsview_decode_option *options, size_t count)
{
    size_t index;

    if (options == NULL) {
        return;
    }

    for (index = 0; index < count; index++) {
        free(options[index].id);
        free(options[index].idn);
        free(options[index].desc);
        free(options[index].default_value);
        dsview_decode_free_string_array(options[index].values, options[index].value_count);
    }
    free(options);
}

static int dsview_decode_option_value_kind_from_variant(const GVariant *value)
{
    GVariantClass variant_class;

    if (value == NULL) {
        return DSVIEW_DECODE_OPTION_VALUE_KIND_UNKNOWN;
    }

    variant_class = g_variant_classify((GVariant *)value);
    switch (variant_class) {
    case G_VARIANT_CLASS_STRING:
        return DSVIEW_DECODE_OPTION_VALUE_KIND_STRING;
    case G_VARIANT_CLASS_BYTE:
    case G_VARIANT_CLASS_INT16:
    case G_VARIANT_CLASS_UINT16:
    case G_VARIANT_CLASS_INT32:
    case G_VARIANT_CLASS_UINT32:
    case G_VARIANT_CLASS_INT64:
    case G_VARIANT_CLASS_UINT64:
        return DSVIEW_DECODE_OPTION_VALUE_KIND_INTEGER;
    case G_VARIANT_CLASS_DOUBLE:
        return DSVIEW_DECODE_OPTION_VALUE_KIND_FLOAT;
    default:
        return DSVIEW_DECODE_OPTION_VALUE_KIND_UNKNOWN;
    }
}

static int dsview_decode_copy_options(
    const GSList *source,
    struct dsview_decode_option **out_options,
    size_t *out_count)
{
    const GSList *node;
    struct dsview_decode_option *options = NULL;
    size_t count = 0;
    size_t index = 0;

    if (out_options == NULL || out_count == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }

    *out_options = NULL;
    *out_count = 0;
    count = (size_t)g_slist_length((GSList *)source);
    if (count == 0) {
        return DSVIEW_DECODE_OK;
    }

    options = (struct dsview_decode_option *)calloc(count, sizeof(*options));
    if (options == NULL) {
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    for (node = source; node != NULL; node = node->next, index++) {
        const struct srd_decoder_option *source_option =
            (const struct srd_decoder_option *)node->data;
        gchar *printed = NULL;

        options[index].id = dsview_decode_strdup(source_option->id);
        options[index].idn = dsview_decode_strdup(source_option->idn);
        options[index].desc = dsview_decode_strdup(source_option->desc);
        options[index].value_kind =
            dsview_decode_option_value_kind_from_variant(source_option->def);
        if (source_option->def != NULL) {
            printed = g_variant_print(source_option->def, TRUE);
            options[index].default_value = dsview_decode_strdup(printed);
            g_free(printed);
        }
        if (source_option->values != NULL) {
            const GSList *value_node;
            size_t value_count = (size_t)g_slist_length(source_option->values);

            options[index].values = (char **)calloc(value_count, sizeof(char *));
            if (options[index].values == NULL) {
                dsview_decode_free_options(options, count);
                return DSVIEW_DECODE_ERR_MALLOC;
            }
            options[index].value_count = value_count;
            for (value_node = source_option->values; value_node != NULL; value_node = value_node->next) {
                gchar *value_text = g_variant_print((GVariant *)value_node->data, TRUE);
                options[index].values[options[index].value_count - value_count] =
                    dsview_decode_strdup(value_text);
                g_free(value_text);
                if (options[index].values[options[index].value_count - value_count] == NULL) {
                    dsview_decode_free_options(options, count);
                    return DSVIEW_DECODE_ERR_MALLOC;
                }
                value_count--;
            }
        }
        if ((source_option->id != NULL && options[index].id == NULL) ||
            (source_option->idn != NULL && options[index].idn == NULL) ||
            (source_option->desc != NULL && options[index].desc == NULL) ||
            (source_option->def != NULL && options[index].default_value == NULL)) {
            dsview_decode_free_options(options, count);
            return DSVIEW_DECODE_ERR_MALLOC;
        }
    }

    *out_options = options;
    *out_count = count;
    return DSVIEW_DECODE_OK;
}

static void dsview_decode_free_annotations(struct dsview_decode_annotation *annotations, size_t count)
{
    size_t index;

    if (annotations == NULL) {
        return;
    }

    for (index = 0; index < count; index++) {
        free(annotations[index].id);
        free(annotations[index].label);
        free(annotations[index].description);
    }
    free(annotations);
}

static int dsview_decode_copy_annotations(
    const GSList *source,
    const GSList *types,
    struct dsview_decode_annotation **out_annotations,
    size_t *out_count)
{
    const GSList *node;
    const GSList *type_node;
    struct dsview_decode_annotation *annotations = NULL;
    size_t count = 0;
    size_t index = 0;

    if (out_annotations == NULL || out_count == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }

    *out_annotations = NULL;
    *out_count = 0;
    count = (size_t)g_slist_length((GSList *)source);
    if (count == 0) {
        return DSVIEW_DECODE_OK;
    }

    annotations = (struct dsview_decode_annotation *)calloc(count, sizeof(*annotations));
    if (annotations == NULL) {
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    type_node = types;
    for (node = source; node != NULL; node = node->next, index++) {
        char **entry = (char **)node->data;
        annotations[index].id = dsview_decode_strdup(entry != NULL ? entry[0] : NULL);
        annotations[index].label = dsview_decode_strdup(entry != NULL ? entry[1] : NULL);
        annotations[index].description = dsview_decode_strdup(entry != NULL ? entry[2] : NULL);
        annotations[index].type =
            type_node != NULL ? GPOINTER_TO_INT(type_node->data) : -1;
        if ((entry != NULL && entry[0] != NULL && annotations[index].id == NULL) ||
            (entry != NULL && entry[1] != NULL && annotations[index].label == NULL) ||
            (entry != NULL && entry[2] != NULL && annotations[index].description == NULL)) {
            dsview_decode_free_annotations(annotations, count);
            return DSVIEW_DECODE_ERR_MALLOC;
        }
        if (type_node != NULL) {
            type_node = type_node->next;
        }
    }

    *out_annotations = annotations;
    *out_count = count;
    return DSVIEW_DECODE_OK;
}

static void dsview_decode_free_annotation_rows(
    struct dsview_decode_annotation_row *rows,
    size_t count)
{
    size_t index;

    if (rows == NULL) {
        return;
    }

    for (index = 0; index < count; index++) {
        free(rows[index].id);
        free(rows[index].desc);
        free(rows[index].annotation_classes);
    }
    free(rows);
}

static int dsview_decode_copy_annotation_rows(
    const GSList *source,
    struct dsview_decode_annotation_row **out_rows,
    size_t *out_count)
{
    const GSList *node;
    struct dsview_decode_annotation_row *rows = NULL;
    size_t count = 0;
    size_t index = 0;

    if (out_rows == NULL || out_count == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }

    *out_rows = NULL;
    *out_count = 0;
    count = (size_t)g_slist_length((GSList *)source);
    if (count == 0) {
        return DSVIEW_DECODE_OK;
    }

    rows = (struct dsview_decode_annotation_row *)calloc(count, sizeof(*rows));
    if (rows == NULL) {
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    for (node = source; node != NULL; node = node->next, index++) {
        const struct srd_decoder_annotation_row *source_row =
            (const struct srd_decoder_annotation_row *)node->data;
        const GSList *class_node;
        size_t class_count = (size_t)g_slist_length(source_row->ann_classes);
        size_t class_index = 0;

        rows[index].id = dsview_decode_strdup(source_row->id);
        rows[index].desc = dsview_decode_strdup(source_row->desc);
        rows[index].annotation_class_count = class_count;
        if ((source_row->id != NULL && rows[index].id == NULL) ||
            (source_row->desc != NULL && rows[index].desc == NULL)) {
            dsview_decode_free_annotation_rows(rows, count);
            return DSVIEW_DECODE_ERR_MALLOC;
        }

        if (class_count > 0) {
            rows[index].annotation_classes =
                (size_t *)calloc(class_count, sizeof(size_t));
            if (rows[index].annotation_classes == NULL) {
                dsview_decode_free_annotation_rows(rows, count);
                return DSVIEW_DECODE_ERR_MALLOC;
            }
        }

        for (class_node = source_row->ann_classes; class_node != NULL; class_node = class_node->next) {
            rows[index].annotation_classes[class_index++] =
                (size_t)GPOINTER_TO_SIZE(class_node->data);
        }
    }

    *out_rows = rows;
    *out_count = count;
    return DSVIEW_DECODE_OK;
}

static void dsview_decode_free_captured_annotations_internal(
    struct dsview_decode_captured_annotation *annotations,
    size_t count)
{
    size_t index;

    if (annotations == NULL) {
        return;
    }

    for (index = 0; index < count; index++) {
        size_t text_index;

        free(annotations[index].decoder_id);
        if (annotations[index].texts != NULL) {
            for (text_index = 0; text_index < annotations[index].text_count; text_index++) {
                free(annotations[index].texts[text_index]);
            }
        }
        free(annotations[index].texts);
    }
    free(annotations);
}

static int dsview_decode_reserve_captured_annotations(
    struct dsview_decode_execution_session *session,
    size_t target_count)
{
    struct dsview_decode_captured_annotation *resized = NULL;
    size_t capacity = 0;

    if (session == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }
    if (target_count <= session->captured_annotation_capacity) {
        return DSVIEW_DECODE_OK;
    }

    capacity = session->captured_annotation_capacity == 0
        ? 8
        : session->captured_annotation_capacity * 2;
    while (capacity < target_count) {
        capacity *= 2;
    }

    resized = (struct dsview_decode_captured_annotation *)realloc(
        session->captured_annotations,
        capacity * sizeof(*resized));
    if (resized == NULL) {
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    memset(
        resized + session->captured_annotation_capacity,
        0,
        (capacity - session->captured_annotation_capacity) * sizeof(*resized));
    session->captured_annotations = resized;
    session->captured_annotation_capacity = capacity;
    return DSVIEW_DECODE_OK;
}

static void dsview_decode_annotation_callback(
    struct srd_proto_data *pdata,
    void *cb_data)
{
    struct dsview_decode_execution_session *session =
        (struct dsview_decode_execution_session *)cb_data;
    struct dsview_decode_captured_annotation *captured = NULL;
    struct srd_proto_data_annotation *annotation = NULL;
    size_t text_count = 0;
    size_t text_index = 0;
    int status;

    if (session == NULL || pdata == NULL || pdata->pdo == NULL) {
        return;
    }

    g_mutex_lock(&session->annotation_lock);
    status = dsview_decode_reserve_captured_annotations(
        session,
        session->captured_annotation_count + 1);
    if (status != DSVIEW_DECODE_OK) {
        g_mutex_unlock(&session->annotation_lock);
        return;
    }

    captured = &session->captured_annotations[session->captured_annotation_count];
    memset(captured, 0, sizeof(*captured));
    captured->decoder_id = dsview_decode_strdup(
        pdata->pdo->di != NULL && pdata->pdo->di->decoder != NULL
            ? pdata->pdo->di->decoder->id
            : pdata->pdo->proto_id);
    if ((pdata->pdo->di != NULL && pdata->pdo->di->decoder != NULL &&
            pdata->pdo->di->decoder->id != NULL &&
            captured->decoder_id == NULL) ||
        (pdata->pdo->di == NULL && pdata->pdo->proto_id != NULL &&
            captured->decoder_id == NULL)) {
        g_mutex_unlock(&session->annotation_lock);
        return;
    }

    captured->start_sample = pdata->start_sample;
    captured->end_sample = pdata->end_sample;
    captured->ann_class = -1;
    captured->ann_type = -1;

    if (pdata->pdo->output_type == SRD_OUTPUT_ANN && pdata->data != NULL) {
        annotation = (struct srd_proto_data_annotation *)pdata->data;
        captured->ann_class = annotation->ann_class;
        captured->ann_type = annotation->ann_type;

        if (annotation->ann_text != NULL) {
            while (annotation->ann_text[text_count] != NULL) {
                text_count++;
            }
        }
        captured->text_count = text_count;
        if (text_count > 0) {
            captured->texts = (char **)calloc(text_count, sizeof(*captured->texts));
            if (captured->texts == NULL) {
                free(captured->decoder_id);
                memset(captured, 0, sizeof(*captured));
                g_mutex_unlock(&session->annotation_lock);
                return;
            }
            for (text_index = 0; text_index < text_count; text_index++) {
                captured->texts[text_index] = dsview_decode_strdup(annotation->ann_text[text_index]);
                if (annotation->ann_text[text_index] != NULL && captured->texts[text_index] == NULL) {
                    size_t cleanup_index;
                    for (cleanup_index = 0; cleanup_index < text_index; cleanup_index++) {
                        free(captured->texts[cleanup_index]);
                    }
                    free(captured->texts);
                    free(captured->decoder_id);
                    memset(captured, 0, sizeof(*captured));
                    g_mutex_unlock(&session->annotation_lock);
                    return;
                }
            }
        }
    }

    session->captured_annotation_count++;
    g_mutex_unlock(&session->annotation_lock);
}

static void dsview_decode_free_list_entries(struct dsview_decode_list_entry *list, size_t count)
{
    size_t index;

    if (list == NULL) {
        return;
    }

    for (index = 0; index < count; index++) {
        free(list[index].id);
        free(list[index].name);
        free(list[index].longname);
        free(list[index].desc);
        free(list[index].license);
    }
    free(list);
}

static int dsview_decode_copy_list_entry(
    const struct srd_decoder *source,
    struct dsview_decode_list_entry *entry)
{
    entry->id = dsview_decode_strdup(source->id);
    entry->name = dsview_decode_strdup(source->name);
    entry->longname = dsview_decode_strdup(source->longname);
    entry->desc = dsview_decode_strdup(source->desc);
    entry->license = dsview_decode_strdup(source->license);

    if ((source->id != NULL && entry->id == NULL) ||
        (source->name != NULL && entry->name == NULL) ||
        (source->longname != NULL && entry->longname == NULL) ||
        (source->desc != NULL && entry->desc == NULL) ||
        (source->license != NULL && entry->license == NULL)) {
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    return DSVIEW_DECODE_OK;
}

int dsview_decode_runtime_load(const char *path)
{
    int status = DSVIEW_DECODE_OK;

    if (path == NULL || path[0] == '\0') {
        return DSVIEW_DECODE_ERR_ARG;
    }

    if (g_decode_runtime_api.library_handle != NULL) {
        return DSVIEW_DECODE_OK;
    }

    dsview_decode_set_loader_error_text(NULL);
    dsview_decode_clear_error_state();

    dsview_bridge_clear_loader_error();
    /*
     * The decode runtime embeds Python and later imports CPython extension
     * modules from decoder dependencies. Those extension modules need the
     * libpython symbols to be globally visible during resolution.
     */
    g_decode_runtime_api.library_handle = dsview_bridge_dlopen_global(path);
    if (g_decode_runtime_api.library_handle == NULL) {
        dsview_decode_set_loader_error_from_loader();
        return DSVIEW_BRIDGE_ERR_DLOPEN;
    }

    g_decode_runtime_api.srd_init =
        (srd_init_fn)dsview_decode_load_symbol("srd_init", &status);
    if (g_decode_runtime_api.srd_init == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_exit =
        (srd_exit_fn)dsview_decode_load_symbol("srd_exit", &status);
    if (g_decode_runtime_api.srd_exit == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_decoder_list =
        (srd_decoder_list_fn)dsview_decode_load_symbol("srd_decoder_list", &status);
    if (g_decode_runtime_api.srd_decoder_list == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_decoder_get_by_id =
        (srd_decoder_get_by_id_fn)dsview_decode_load_symbol("srd_decoder_get_by_id", &status);
    if (g_decode_runtime_api.srd_decoder_get_by_id == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_decoder_load_all =
        (srd_decoder_load_all_fn)dsview_decode_load_symbol("srd_decoder_load_all", &status);
    if (g_decode_runtime_api.srd_decoder_load_all == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_searchpaths_get =
        (srd_searchpaths_get_fn)dsview_decode_load_symbol("srd_searchpaths_get", &status);
    if (g_decode_runtime_api.srd_searchpaths_get == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_strerror =
        (srd_strerror_fn)dsview_decode_load_symbol("srd_strerror", &status);
    if (g_decode_runtime_api.srd_strerror == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_strerror_name =
        (srd_strerror_name_fn)dsview_decode_load_symbol("srd_strerror_name", &status);
    if (g_decode_runtime_api.srd_strerror_name == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_session_new =
        (srd_session_new_fn)dsview_decode_load_symbol("srd_session_new", &status);
    if (g_decode_runtime_api.srd_session_new == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_session_metadata_set =
        (srd_session_metadata_set_fn)dsview_decode_load_symbol("srd_session_metadata_set", &status);
    if (g_decode_runtime_api.srd_session_metadata_set == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_session_start =
        (srd_session_start_fn)dsview_decode_load_symbol("srd_session_start", &status);
    if (g_decode_runtime_api.srd_session_start == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_session_send =
        (srd_session_send_fn)dsview_decode_load_symbol("srd_session_send", &status);
    if (g_decode_runtime_api.srd_session_send == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_session_end =
        (srd_session_end_fn)dsview_decode_load_symbol("srd_session_end", &status);
    if (g_decode_runtime_api.srd_session_end == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_session_destroy =
        (srd_session_destroy_fn)dsview_decode_load_symbol("srd_session_destroy", &status);
    if (g_decode_runtime_api.srd_session_destroy == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_pd_output_callback_add =
        (srd_pd_output_callback_add_fn)dsview_decode_load_symbol("srd_pd_output_callback_add", &status);
    if (g_decode_runtime_api.srd_pd_output_callback_add == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_inst_new =
        (srd_inst_new_fn)dsview_decode_load_symbol("srd_inst_new", &status);
    if (g_decode_runtime_api.srd_inst_new == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_inst_channel_set_all =
        (srd_inst_channel_set_all_fn)dsview_decode_load_symbol("srd_inst_channel_set_all", &status);
    if (g_decode_runtime_api.srd_inst_channel_set_all == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }
    g_decode_runtime_api.srd_inst_stack =
        (srd_inst_stack_fn)dsview_decode_load_symbol("srd_inst_stack", &status);
    if (g_decode_runtime_api.srd_inst_stack == NULL) {
        dsview_decode_runtime_unload();
        return status;
    }

    return DSVIEW_DECODE_OK;
}

void dsview_decode_runtime_unload(void)
{
    if (g_decode_runtime_api.initialized && g_decode_runtime_api.srd_exit != NULL) {
        g_decode_runtime_api.srd_exit();
    }
    if (g_decode_runtime_api.library_handle != NULL) {
        dsview_bridge_dlclose(g_decode_runtime_api.library_handle);
    }
    memset(&g_decode_runtime_api, 0, sizeof(g_decode_runtime_api));
}

int dsview_decode_runtime_init(const char *decoder_dir)
{
    const GSList *decoders;
    int status;

    if (g_decode_runtime_api.library_handle == NULL) {
        return dsview_decode_not_loaded();
    }
    if (decoder_dir == NULL || decoder_dir[0] == '\0') {
        return DSVIEW_DECODE_ERR_ARG;
    }
    if (!g_file_test(decoder_dir, G_FILE_TEST_IS_DIR)) {
        dsview_decode_set_error_detail(
            DSVIEW_DECODE_ERR_DECODER_DIR,
            "decoder directory missing",
            "decode runtime init requires an explicit decoder directory that exists");
        return DSVIEW_DECODE_ERR_DECODER_DIR;
    }

    if (g_decode_runtime_api.initialized && g_decode_runtime_api.srd_exit != NULL) {
        g_decode_runtime_api.srd_exit();
        g_decode_runtime_api.initialized = 0;
    }

    dsview_decode_clear_error_state();
    status = g_decode_runtime_api.srd_init(decoder_dir);
    if (status != SRD_OK) {
        dsview_decode_capture_upstream_error(status, 0);
        return dsview_decode_map_upstream_status(status, 0);
    }
    g_decode_runtime_api.initialized = 1;

    status = g_decode_runtime_api.srd_decoder_load_all();
    if (status != SRD_OK) {
        dsview_decode_capture_upstream_error(status, 1);
        g_decode_runtime_api.srd_exit();
        g_decode_runtime_api.initialized = 0;
        return dsview_decode_map_upstream_status(status, 1);
    }

    decoders = g_decode_runtime_api.srd_decoder_list();
    if (decoders == NULL || g_slist_length((GSList *)decoders) == 0) {
        dsview_decode_set_error_detail(
            DSVIEW_DECODE_ERR_DECODER_LOAD,
            "decoder load failed",
            "decode runtime init completed but no decoders were loaded");
        g_decode_runtime_api.srd_exit();
        g_decode_runtime_api.initialized = 0;
        return DSVIEW_DECODE_ERR_DECODER_LOAD;
    }

    return DSVIEW_DECODE_OK;
}

int dsview_decode_runtime_exit(void)
{
    int status;

    if (g_decode_runtime_api.library_handle == NULL) {
        return dsview_decode_not_loaded();
    }
    if (!g_decode_runtime_api.initialized) {
        return DSVIEW_DECODE_OK;
    }

    status = g_decode_runtime_api.srd_exit();
    if (status != SRD_OK) {
        dsview_decode_capture_upstream_error(status, 0);
        return dsview_decode_map_upstream_status(status, 0);
    }

    g_decode_runtime_api.initialized = 0;
    dsview_decode_clear_error_state();
    return DSVIEW_DECODE_OK;
}

const char *dsview_decode_last_loader_error(void)
{
    return g_decode_runtime_api.last_loader_error;
}

const char *dsview_decode_last_error(void)
{
    return g_decode_runtime_api.last_error;
}

const char *dsview_decode_last_error_name(void)
{
    return g_decode_runtime_api.last_error_name;
}

int dsview_decode_list(struct dsview_decode_list_entry **out_list, size_t *out_count)
{
    const GSList *node;
    const GSList *decoders;
    struct dsview_decode_list_entry *list = NULL;
    size_t count = 0;
    size_t index = 0;
    int status;

    if (out_list == NULL || out_count == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }
    *out_list = NULL;
    *out_count = 0;

    if (g_decode_runtime_api.library_handle == NULL) {
        return dsview_decode_not_loaded();
    }
    if (!g_decode_runtime_api.initialized) {
        return dsview_decode_not_initialized();
    }

    decoders = g_decode_runtime_api.srd_decoder_list();
    count = (size_t)g_slist_length((GSList *)decoders);
    if (count == 0) {
        return DSVIEW_DECODE_OK;
    }

    list = (struct dsview_decode_list_entry *)calloc(count, sizeof(*list));
    if (list == NULL) {
        dsview_decode_set_error_detail(
            DSVIEW_DECODE_ERR_MALLOC,
            "memory allocation error",
            "failed to allocate decode_list snapshot");
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    for (node = decoders; node != NULL; node = node->next, index++) {
        status = dsview_decode_copy_list_entry((const struct srd_decoder *)node->data, &list[index]);
        if (status != DSVIEW_DECODE_OK) {
            dsview_decode_free_list_entries(list, count);
            dsview_decode_set_error_detail(
                status,
                "memory allocation error",
                "failed to copy decode_list snapshot");
            return status;
        }
    }

    *out_list = list;
    *out_count = count;
    return DSVIEW_DECODE_OK;
}

void dsview_decode_free_list(struct dsview_decode_list_entry *list, size_t count)
{
    dsview_decode_free_list_entries(list, count);
}

void dsview_decode_free_metadata(struct dsview_decode_metadata *metadata)
{
    if (metadata == NULL) {
        return;
    }

    free(metadata->id);
    free(metadata->name);
    free(metadata->longname);
    free(metadata->desc);
    free(metadata->license);
    dsview_decode_free_string_array(metadata->inputs, metadata->input_count);
    dsview_decode_free_string_array(metadata->outputs, metadata->output_count);
    dsview_decode_free_string_array(metadata->tags, metadata->tag_count);
    dsview_decode_free_channels(metadata->required_channels, metadata->required_channel_count);
    dsview_decode_free_channels(metadata->optional_channels, metadata->optional_channel_count);
    dsview_decode_free_options(metadata->options, metadata->option_count);
    dsview_decode_free_annotations(metadata->annotations, metadata->annotation_count);
    dsview_decode_free_annotation_rows(metadata->annotation_rows, metadata->annotation_row_count);
    memset(metadata, 0, sizeof(*metadata));
}

static GHashTable *dsview_decode_build_option_table(
    const struct dsview_decode_option_entry *options,
    size_t option_count)
{
    GHashTable *table = NULL;
    size_t index;

    table = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, (GDestroyNotify)g_variant_unref);
    if (table == NULL) {
        return NULL;
    }

    for (index = 0; index < option_count; index++) {
        GVariant *value = NULL;

        if (options[index].option_id == NULL || options[index].option_id[0] == '\0') {
            g_hash_table_destroy(table);
            dsview_decode_invalid_session("decode option ids must not be empty");
            return NULL;
        }

        switch (options[index].value.kind) {
        case DSVIEW_DECODE_OPTION_VALUE_KIND_STRING:
            if (options[index].value.string_value == NULL) {
                g_hash_table_destroy(table);
                dsview_decode_invalid_session("string decode option values must not be null");
                return NULL;
            }
            value = g_variant_new_string(options[index].value.string_value);
            break;
        case DSVIEW_DECODE_OPTION_VALUE_KIND_INTEGER:
            value = g_variant_new_int64((gint64)options[index].value.integer_value);
            break;
        case DSVIEW_DECODE_OPTION_VALUE_KIND_FLOAT:
            value = g_variant_new_double(options[index].value.float_value);
            break;
        default:
            g_hash_table_destroy(table);
            dsview_decode_invalid_session("unsupported decode option value kind");
            return NULL;
        }

        g_hash_table_insert(table, g_strdup(options[index].option_id), g_variant_ref_sink(value));
    }

    return table;
}

static GHashTable *dsview_decode_build_channel_table(
    const struct dsview_decode_channel_binding *bindings,
    size_t binding_count,
    unsigned int *out_input_channel_count)
{
    GHashTable *table = NULL;
    size_t index;
    unsigned int input_channel_count = 0;

    table = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, (GDestroyNotify)g_variant_unref);
    if (table == NULL) {
        return NULL;
    }

    for (index = 0; index < binding_count; index++) {
        if (bindings[index].channel_id == NULL || bindings[index].channel_id[0] == '\0') {
            g_hash_table_destroy(table);
            dsview_decode_invalid_session("decode channel ids must not be empty");
            return NULL;
        }

        if (bindings[index].channel_index >= DSVIEW_BRIDGE_CHANNEL_TRACK_CAPACITY) {
            g_hash_table_destroy(table);
            dsview_decode_invalid_session("decode channel bindings exceed supported input channel capacity");
            return NULL;
        }

        g_hash_table_insert(
            table,
            g_strdup(bindings[index].channel_id),
            g_variant_ref_sink(g_variant_new_int32((gint32)bindings[index].channel_index)));

        if (bindings[index].channel_index + 1 > input_channel_count) {
            input_channel_count = bindings[index].channel_index + 1;
        }
    }

    if (out_input_channel_count != NULL) {
        *out_input_channel_count = input_channel_count;
    }

    return table;
}

static struct srd_decoder_inst *dsview_decode_create_instance(
    struct dsview_decode_execution_session *session,
    const struct dsview_decode_instance_spec *spec,
    int allow_channel_bindings,
    unsigned int *out_input_channel_count)
{
    GHashTable *options = NULL;
    GHashTable *channels = NULL;
    struct srd_decoder_inst *instance = NULL;
    unsigned int input_channel_count = 0;
    int status;

    if (session == NULL || spec == NULL || spec->decoder_id == NULL || spec->decoder_id[0] == '\0') {
        dsview_decode_invalid_session("decode instance spec must include a decoder id");
        return NULL;
    }

    if (!allow_channel_bindings && spec->channel_binding_count > 0) {
        dsview_decode_invalid_session("only the root decoder may bind logic channels");
        return NULL;
    }

    options = dsview_decode_build_option_table(spec->options, spec->option_count);
    if (spec->option_count > 0 && options == NULL) {
        return NULL;
    }

    instance = g_decode_runtime_api.srd_inst_new(session->session, spec->decoder_id, options);
    if (options != NULL) {
        g_hash_table_destroy(options);
    }
    if (instance == NULL) {
        dsview_decode_set_error_detail(
            DSVIEW_DECODE_ERR_UPSTREAM,
            "decoder instance creation failed",
            "failed to create decode runtime instance");
        return NULL;
    }

    if (allow_channel_bindings && spec->channel_binding_count > 0) {
        channels = dsview_decode_build_channel_table(
            spec->channel_bindings,
            spec->channel_binding_count,
            &input_channel_count);
        if (channels == NULL) {
            return NULL;
        }

        status = g_decode_runtime_api.srd_inst_channel_set_all(instance, channels);
        g_hash_table_destroy(channels);
        if (status != SRD_OK) {
            dsview_decode_capture_upstream_error(status, 0);
            return NULL;
        }
    }

    if (out_input_channel_count != NULL) {
        *out_input_channel_count = input_channel_count;
    }
    return instance;
}

static int dsview_decode_build_split_chunk(
    const struct dsview_decode_execution_session *session,
    const struct dsview_decode_logic_chunk *chunk,
    const uint8_t ***out_inbuf,
    uint8_t **out_inbuf_const,
    uint8_t ***out_owned_buffers,
    uint64_t *out_sample_count)
{
    const uint8_t **inbuf = NULL;
    uint8_t *inbuf_const = NULL;
    uint8_t **owned_buffers = NULL;
    size_t sample_count;
    size_t root_channel_count;
    size_t buffer_len;
    size_t sample_index;
    int channel_order;

    if (chunk->unitsize == 0) {
        return dsview_decode_invalid_shape("split-logic chunks require a non-zero unitsize");
    }
    if ((chunk->sample_bytes_len % chunk->unitsize) != 0) {
        return dsview_decode_invalid_shape("split-logic sample bytes must align to unitsize");
    }
    if (session->input_channel_count == 0) {
        return dsview_decode_invalid_session("decode session is missing root channel bindings");
    }
    if (((unsigned int)chunk->unitsize * 8U) < session->input_channel_count) {
        return dsview_decode_invalid_shape("split-logic unitsize cannot represent the bound channel indexes");
    }

    sample_count = chunk->sample_bytes_len / chunk->unitsize;
    if (sample_count == 0) {
        return dsview_decode_invalid_shape("split-logic chunks must contain at least one sample");
    }

    root_channel_count = (size_t)session->root->dec_num_channels;
    buffer_len = (sample_count + 7U) / 8U;

    inbuf = (const uint8_t **)calloc(root_channel_count, sizeof(*inbuf));
    inbuf_const = (uint8_t *)calloc(root_channel_count, sizeof(*inbuf_const));
    owned_buffers = (uint8_t **)calloc(root_channel_count, sizeof(*owned_buffers));
    if (inbuf == NULL || inbuf_const == NULL || owned_buffers == NULL) {
        free((void *)inbuf);
        free(inbuf_const);
        free(owned_buffers);
        dsview_decode_set_error_detail(
            DSVIEW_DECODE_ERR_MALLOC,
            "memory allocation error",
            "failed to allocate split-logic decode buffers");
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    for (channel_order = 0; channel_order < session->root->dec_num_channels; channel_order++) {
        int raw_channel_index = session->root->dec_channelmap[channel_order];

        if (raw_channel_index < 0) {
            inbuf[channel_order] = NULL;
            inbuf_const[channel_order] = 0;
            continue;
        }

        owned_buffers[channel_order] = (uint8_t *)calloc(buffer_len, sizeof(uint8_t));
        if (owned_buffers[channel_order] == NULL) {
            dsview_decode_free_chunk_buffers(session, inbuf, inbuf_const, owned_buffers);
            dsview_decode_set_error_detail(
                DSVIEW_DECODE_ERR_MALLOC,
                "memory allocation error",
                "failed to allocate split-logic per-channel buffers");
            return DSVIEW_DECODE_ERR_MALLOC;
        }

        for (sample_index = 0; sample_index < sample_count; sample_index++) {
            const uint8_t *sample = chunk->sample_bytes + (sample_index * chunk->unitsize);
            uint8_t sample_byte = sample[raw_channel_index / 8];
            uint8_t bit = (uint8_t)((sample_byte >> (raw_channel_index % 8)) & 0x1U);

            if (bit != 0) {
                owned_buffers[channel_order][sample_index / 8] |= (uint8_t)(1U << (sample_index % 8));
            }
        }

        inbuf[channel_order] = owned_buffers[channel_order];
    }

    *out_inbuf = inbuf;
    *out_inbuf_const = inbuf_const;
    *out_owned_buffers = owned_buffers;
    *out_sample_count = (uint64_t)sample_count;
    return DSVIEW_DECODE_OK;
}

static int dsview_decode_build_cross_chunk(
    const struct dsview_decode_execution_session *session,
    const struct dsview_decode_logic_chunk *chunk,
    const uint8_t ***out_inbuf,
    uint8_t **out_inbuf_const,
    uint8_t ***out_owned_buffers,
    uint64_t *out_sample_count)
{
    const uint8_t **inbuf = NULL;
    uint8_t *inbuf_const = NULL;
    uint8_t **owned_buffers = NULL;
    size_t block_count;
    size_t root_channel_count;
    size_t channel_bytes;
    size_t block_stride;
    size_t block_index;
    int channel_order;

    if (chunk->channel_count == 0) {
        return dsview_decode_invalid_shape("cross-logic chunks require a non-zero channel count");
    }
    if (session->input_channel_count == 0) {
        return dsview_decode_invalid_session("decode session is missing root channel bindings");
    }
    if (chunk->channel_count < session->input_channel_count) {
        return dsview_decode_invalid_shape("cross-logic channel count must cover every bound root channel");
    }

    block_stride = (size_t)chunk->channel_count * sizeof(uint64_t);
    if ((chunk->sample_bytes_len % block_stride) != 0) {
        return dsview_decode_invalid_shape("cross-logic sample bytes must align to channel_count * 8");
    }

    block_count = chunk->sample_bytes_len / block_stride;
    if (block_count == 0) {
        return dsview_decode_invalid_shape("cross-logic chunks must contain at least one sample block");
    }

    root_channel_count = (size_t)session->root->dec_num_channels;
    channel_bytes = block_count * sizeof(uint64_t);

    inbuf = (const uint8_t **)calloc(root_channel_count, sizeof(*inbuf));
    inbuf_const = (uint8_t *)calloc(root_channel_count, sizeof(*inbuf_const));
    owned_buffers = (uint8_t **)calloc(root_channel_count, sizeof(*owned_buffers));
    if (inbuf == NULL || inbuf_const == NULL || owned_buffers == NULL) {
        free((void *)inbuf);
        free(inbuf_const);
        free(owned_buffers);
        dsview_decode_set_error_detail(
            DSVIEW_DECODE_ERR_MALLOC,
            "memory allocation error",
            "failed to allocate cross-logic decode buffers");
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    for (channel_order = 0; channel_order < session->root->dec_num_channels; channel_order++) {
        int raw_channel_index = session->root->dec_channelmap[channel_order];

        if (raw_channel_index < 0) {
            inbuf[channel_order] = NULL;
            inbuf_const[channel_order] = 0;
            continue;
        }

        owned_buffers[channel_order] = (uint8_t *)malloc(channel_bytes);
        if (owned_buffers[channel_order] == NULL) {
            dsview_decode_free_chunk_buffers(session, inbuf, inbuf_const, owned_buffers);
            dsview_decode_set_error_detail(
                DSVIEW_DECODE_ERR_MALLOC,
                "memory allocation error",
                "failed to allocate cross-logic per-channel buffers");
            return DSVIEW_DECODE_ERR_MALLOC;
        }

        for (block_index = 0; block_index < block_count; block_index++) {
            const uint8_t *src = chunk->sample_bytes + (block_index * block_stride) + ((size_t)raw_channel_index * sizeof(uint64_t));
            memcpy(owned_buffers[channel_order] + (block_index * sizeof(uint64_t)), src, sizeof(uint64_t));
        }

        inbuf[channel_order] = owned_buffers[channel_order];
    }

    *out_inbuf = inbuf;
    *out_inbuf_const = inbuf_const;
    *out_owned_buffers = owned_buffers;
    *out_sample_count = (uint64_t)(block_count * 64U);
    return DSVIEW_DECODE_OK;
}

static void dsview_decode_free_chunk_buffers(
    const struct dsview_decode_execution_session *session,
    const uint8_t **inbuf,
    uint8_t *inbuf_const,
    uint8_t **owned_buffers)
{
    int channel_order;

    if (owned_buffers != NULL && session != NULL && session->root != NULL) {
        for (channel_order = 0; channel_order < session->root->dec_num_channels; channel_order++) {
            free(owned_buffers[channel_order]);
        }
    }

    free(owned_buffers);
    free(inbuf_const);
    free((void *)inbuf);
}

int dsview_decode_session_new(struct dsview_decode_execution_session **out_session)
{
    struct dsview_decode_execution_session *session = NULL;
    int status;

    if (out_session == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }
    *out_session = NULL;

    if (g_decode_runtime_api.library_handle == NULL) {
        return dsview_decode_not_loaded();
    }
    if (!g_decode_runtime_api.initialized) {
        return dsview_decode_not_initialized();
    }

    session = (struct dsview_decode_execution_session *)calloc(1, sizeof(*session));
    if (session == NULL) {
        dsview_decode_set_error_detail(
            DSVIEW_DECODE_ERR_MALLOC,
            "memory allocation error",
            "failed to allocate decode execution session");
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    status = g_decode_runtime_api.srd_session_new(&session->session);
    if (status != SRD_OK || session->session == NULL) {
        free(session);
        dsview_decode_capture_upstream_error(status, 0);
        return dsview_decode_map_upstream_status(status, 0);
    }

    g_mutex_init(&session->annotation_lock);
    status = g_decode_runtime_api.srd_pd_output_callback_add(
        session->session,
        SRD_OUTPUT_ANN,
        dsview_decode_annotation_callback,
        session);
    if (status != SRD_OK) {
        g_mutex_clear(&session->annotation_lock);
        g_decode_runtime_api.srd_session_destroy(session->session);
        free(session);
        dsview_decode_capture_upstream_error(status, 0);
        return dsview_decode_map_upstream_status(status, 0);
    }
    status = g_decode_runtime_api.srd_pd_output_callback_add(
        session->session,
        SRD_OUTPUT_PYTHON,
        dsview_decode_annotation_callback,
        session);
    if (status != SRD_OK) {
        g_mutex_clear(&session->annotation_lock);
        g_decode_runtime_api.srd_session_destroy(session->session);
        free(session);
        dsview_decode_capture_upstream_error(status, 0);
        return dsview_decode_map_upstream_status(status, 0);
    }

    dsview_decode_clear_error_state();
    *out_session = session;
    return DSVIEW_DECODE_OK;
}

int dsview_decode_session_set_samplerate(
    struct dsview_decode_execution_session *session,
    unsigned long long samplerate_hz)
{
    int status;

    if (session == NULL || session->session == NULL) {
        return dsview_decode_invalid_session("decode session has not been created");
    }
    if (samplerate_hz == 0) {
        return DSVIEW_DECODE_ERR_ARG;
    }

    status = g_decode_runtime_api.srd_session_metadata_set(
        session->session,
        SRD_CONF_SAMPLERATE,
        g_variant_new_uint64((guint64)samplerate_hz));
    if (status != SRD_OK) {
        dsview_decode_capture_upstream_error(status, 0);
        return dsview_decode_map_upstream_status(status, 0);
    }

    dsview_decode_clear_error_state();
    return DSVIEW_DECODE_OK;
}

int dsview_decode_session_build_linear_stack(
    struct dsview_decode_execution_session *session,
    const struct dsview_decode_instance_spec *root,
    const struct dsview_decode_instance_spec *stack,
    size_t stack_count)
{
    struct srd_decoder_inst *previous = NULL;
    size_t index;
    int status;

    if (session == NULL || session->session == NULL || root == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }
    if (session->root != NULL) {
        return dsview_decode_invalid_session("decode session stack has already been built");
    }

    session->root = dsview_decode_create_instance(session, root, 1, &session->input_channel_count);
    if (session->root == NULL) {
        return g_decode_runtime_api.last_error_code != DSVIEW_DECODE_OK
            ? g_decode_runtime_api.last_error_code
            : DSVIEW_DECODE_ERR_UPSTREAM;
    }

    previous = session->root;
    for (index = 0; index < stack_count; index++) {
        struct srd_decoder_inst *current =
            dsview_decode_create_instance(session, &stack[index], 0, NULL);
        if (current == NULL) {
            return g_decode_runtime_api.last_error_code != DSVIEW_DECODE_OK
                ? g_decode_runtime_api.last_error_code
                : DSVIEW_DECODE_ERR_UPSTREAM;
        }

        status = g_decode_runtime_api.srd_inst_stack(session->session, previous, current);
        if (status != SRD_OK) {
            dsview_decode_capture_upstream_error(status, 0);
            return dsview_decode_map_upstream_status(status, 0);
        }

        previous = current;
    }

    dsview_decode_clear_error_state();
    return DSVIEW_DECODE_OK;
}

int dsview_decode_session_start(struct dsview_decode_execution_session *session)
{
    int status;
    char *error = NULL;

    if (session == NULL || session->session == NULL) {
        return dsview_decode_invalid_session("decode session has not been created");
    }
    if (session->root == NULL) {
        return dsview_decode_invalid_session("decode session must build a root decoder before start");
    }
    if (session->started) {
        return dsview_decode_invalid_session("decode session has already been started");
    }

    status = g_decode_runtime_api.srd_session_start(session->session, &error);
    if (status != SRD_OK) {
        if (error != NULL) {
            dsview_decode_clear_error_from_gchar(error);
        } else {
            dsview_decode_capture_upstream_error(status, 0);
        }
        return dsview_decode_map_upstream_status(status, 0);
    }

    session->started = 1;
    session->ended = 0;
    dsview_decode_clear_error_state();
    return DSVIEW_DECODE_OK;
}

int dsview_decode_session_send_logic_chunk(
    struct dsview_decode_execution_session *session,
    const struct dsview_decode_logic_chunk *chunk)
{
    const uint8_t **inbuf = NULL;
    uint8_t *inbuf_const = NULL;
    uint8_t **owned_buffers = NULL;
    uint64_t sample_count = 0;
    int status;
    char *error = NULL;

    if (session == NULL || session->session == NULL || chunk == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }
    if (!session->started || session->ended) {
        return dsview_decode_invalid_session("decode session must be started before sending logic chunks");
    }
    if (chunk->sample_bytes == NULL || chunk->sample_bytes_len == 0) {
        return dsview_decode_invalid_shape("logic chunks must include sample bytes");
    }
    if (chunk->abs_end_sample <= chunk->abs_start_sample) {
        return dsview_decode_invalid_shape("logic chunk absolute bounds must advance forward");
    }

    switch (chunk->format) {
    case DSVIEW_DECODE_LOGIC_FORMAT_SPLIT:
        status = dsview_decode_build_split_chunk(
            session,
            chunk,
            &inbuf,
            &inbuf_const,
            &owned_buffers,
            &sample_count);
        break;
    case DSVIEW_DECODE_LOGIC_FORMAT_CROSS:
        status = dsview_decode_build_cross_chunk(
            session,
            chunk,
            &inbuf,
            &inbuf_const,
            &owned_buffers,
            &sample_count);
        break;
    default:
        return dsview_decode_invalid_shape("unknown decode logic chunk format");
    }

    if (status != DSVIEW_DECODE_OK) {
        return status;
    }

    if ((chunk->abs_end_sample - chunk->abs_start_sample) != sample_count) {
        dsview_decode_free_chunk_buffers(session, inbuf, inbuf_const, owned_buffers);
        return dsview_decode_invalid_shape("logic chunk absolute bounds must match the decoded sample count");
    }

    status = g_decode_runtime_api.srd_session_send(
        session->session,
        chunk->abs_start_sample,
        chunk->abs_end_sample,
        inbuf,
        inbuf_const,
        sample_count,
        &error);
    dsview_decode_free_chunk_buffers(session, inbuf, inbuf_const, owned_buffers);
    if (status != SRD_OK) {
        if (error != NULL) {
            dsview_decode_clear_error_from_gchar(error);
        } else {
            dsview_decode_capture_upstream_error(status, 0);
        }
        return dsview_decode_map_upstream_status(status, 0);
    }

    dsview_decode_clear_error_state();
    return DSVIEW_DECODE_OK;
}

int dsview_decode_session_end(struct dsview_decode_execution_session *session)
{
    int status;
    char *error = NULL;

    if (session == NULL || session->session == NULL) {
        return dsview_decode_invalid_session("decode session has not been created");
    }
    if (!session->started) {
        return dsview_decode_invalid_session("decode session must be started before it can end");
    }
    if (session->ended) {
        return DSVIEW_DECODE_OK;
    }

    status = g_decode_runtime_api.srd_session_end(session->session, &error);
    if (status != SRD_OK) {
        if (error != NULL) {
            dsview_decode_clear_error_from_gchar(error);
        } else {
            dsview_decode_capture_upstream_error(status, 0);
        }
        return dsview_decode_map_upstream_status(status, 0);
    }

    session->ended = 1;
    dsview_decode_clear_error_state();
    return DSVIEW_DECODE_OK;
}

int dsview_decode_session_take_captured_annotations(
    struct dsview_decode_execution_session *session,
    struct dsview_decode_captured_annotation **out_annotations,
    size_t *out_count)
{
    if (session == NULL || out_annotations == NULL || out_count == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }

    g_mutex_lock(&session->annotation_lock);
    *out_annotations = session->captured_annotations;
    *out_count = session->captured_annotation_count;
    session->captured_annotations = NULL;
    session->captured_annotation_count = 0;
    session->captured_annotation_capacity = 0;
    g_mutex_unlock(&session->annotation_lock);

    return DSVIEW_DECODE_OK;
}

void dsview_decode_free_captured_annotations(
    struct dsview_decode_captured_annotation *annotations,
    size_t count)
{
    dsview_decode_free_captured_annotations_internal(annotations, count);
}

void dsview_decode_session_destroy(struct dsview_decode_execution_session *session)
{
    if (session == NULL) {
        return;
    }

    dsview_decode_free_captured_annotations_internal(
        session->captured_annotations,
        session->captured_annotation_count);
    g_mutex_clear(&session->annotation_lock);

    if (session->session != NULL && g_decode_runtime_api.srd_session_destroy != NULL) {
        g_decode_runtime_api.srd_session_destroy(session->session);
    }

    free(session);
}

int dsview_decode_inspect(const char *decoder_id, struct dsview_decode_metadata *out_metadata)
{
    const struct srd_decoder *decoder = NULL;
    int status;

    if (decoder_id == NULL || out_metadata == NULL) {
        return DSVIEW_DECODE_ERR_ARG;
    }
    memset(out_metadata, 0, sizeof(*out_metadata));

    if (g_decode_runtime_api.library_handle == NULL) {
        return dsview_decode_not_loaded();
    }
    if (!g_decode_runtime_api.initialized) {
        return dsview_decode_not_initialized();
    }

    decoder = g_decode_runtime_api.srd_decoder_get_by_id(decoder_id);
    if (decoder == NULL) {
        dsview_decode_set_error_detail(
            DSVIEW_DECODE_ERR_UNKNOWN_DECODER,
            "unknown decoder",
            "decode inspect requested an unknown decoder id");
        return DSVIEW_DECODE_ERR_UNKNOWN_DECODER;
    }

    out_metadata->id = dsview_decode_strdup(decoder->id);
    out_metadata->name = dsview_decode_strdup(decoder->name);
    out_metadata->longname = dsview_decode_strdup(decoder->longname);
    out_metadata->desc = dsview_decode_strdup(decoder->desc);
    out_metadata->license = dsview_decode_strdup(decoder->license);
    if ((decoder->id != NULL && out_metadata->id == NULL) ||
        (decoder->name != NULL && out_metadata->name == NULL) ||
        (decoder->longname != NULL && out_metadata->longname == NULL) ||
        (decoder->desc != NULL && out_metadata->desc == NULL) ||
        (decoder->license != NULL && out_metadata->license == NULL)) {
        dsview_decode_free_metadata(out_metadata);
        dsview_decode_set_error_detail(
            DSVIEW_DECODE_ERR_MALLOC,
            "memory allocation error",
            "failed to allocate decode_inspect metadata");
        return DSVIEW_DECODE_ERR_MALLOC;
    }

    status = dsview_decode_copy_string_array(decoder->inputs, &out_metadata->inputs, &out_metadata->input_count);
    if (status != DSVIEW_DECODE_OK) goto fail;
    status = dsview_decode_copy_string_array(decoder->outputs, &out_metadata->outputs, &out_metadata->output_count);
    if (status != DSVIEW_DECODE_OK) goto fail;
    status = dsview_decode_copy_string_array(decoder->tags, &out_metadata->tags, &out_metadata->tag_count);
    if (status != DSVIEW_DECODE_OK) goto fail;
    status = dsview_decode_copy_channels(decoder->channels, &out_metadata->required_channels, &out_metadata->required_channel_count);
    if (status != DSVIEW_DECODE_OK) goto fail;
    status = dsview_decode_copy_channels(decoder->opt_channels, &out_metadata->optional_channels, &out_metadata->optional_channel_count);
    if (status != DSVIEW_DECODE_OK) goto fail;
    status = dsview_decode_copy_options(decoder->options, &out_metadata->options, &out_metadata->option_count);
    if (status != DSVIEW_DECODE_OK) goto fail;
    status = dsview_decode_copy_annotations(decoder->annotations, decoder->ann_types, &out_metadata->annotations, &out_metadata->annotation_count);
    if (status != DSVIEW_DECODE_OK) goto fail;
    status = dsview_decode_copy_annotation_rows(decoder->annotation_rows, &out_metadata->annotation_rows, &out_metadata->annotation_row_count);
    if (status != DSVIEW_DECODE_OK) goto fail;

    return DSVIEW_DECODE_OK;

fail:
    dsview_decode_free_metadata(out_metadata);
    dsview_decode_set_error_detail(
        status,
        "memory allocation error",
        "failed to copy decode_inspect metadata");
    return status;
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

static int dsview_bridge_get_optional_int16_config(int key, int *has_value, int *value)
{
    GVariant *data = NULL;
    int status;

    if (has_value == NULL || value == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (g_bridge_api.ds_get_actived_device_config == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    *has_value = 0;
    *value = 0;

    status = g_bridge_api.ds_get_actived_device_config(NULL, NULL, key, &data);
    if (status == SR_ERR_NA) {
        return SR_OK;
    }
    if (status != SR_OK) {
        return status;
    }
    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    *value = g_variant_get_int16(data);
    *has_value = 1;
    g_variant_unref(data);
    return SR_OK;
}

static int dsview_bridge_get_optional_double_config(int key, int *has_value, double *value)
{
    GVariant *data = NULL;
    int status;

    if (has_value == NULL || value == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (g_bridge_api.ds_get_actived_device_config == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    *has_value = 0;
    *value = 0.0;

    status = g_bridge_api.ds_get_actived_device_config(NULL, NULL, key, &data);
    if (status == SR_ERR_NA) {
        return SR_OK;
    }
    if (status != SR_OK) {
        return status;
    }
    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    *value = g_variant_get_double(data);
    *has_value = 1;
    g_variant_unref(data);
    return SR_OK;
}

static int dsview_bridge_set_int16_config(int key, int value)
{
    GVariant *data;

    if (g_bridge_api.ds_set_actived_device_config == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    data = g_variant_new_int16((gint16)value);
    return g_bridge_api.ds_set_actived_device_config(NULL, NULL, key, data);
}

static void dsview_bridge_copy_string(char *dst, size_t dst_len, const char *src)
{
    if (dst == NULL || dst_len == 0) {
        return;
    }

    memset(dst, 0, dst_len);
    if (src != NULL) {
        strncpy(dst, src, dst_len - 1);
    }
}

static int dsview_bridge_copy_option_values(
    int key,
    struct dsview_option_value *out_values,
    int max_values,
    unsigned short *out_count)
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
    status = g_bridge_api.ds_get_actived_device_config_list(NULL, key, &data);
    if (status == SR_ERR_NA) {
        return SR_OK;
    }
    if (status != SR_OK) {
        return status;
    }
    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    items = (struct sr_list_item *)(uintptr_t)g_variant_get_uint64(data);
    while (items != NULL && items[index].id >= 0) {
        if (out_values != NULL && index < max_values) {
            out_values[index].code = items[index].id;
            dsview_bridge_copy_string(
                out_values[index].label,
                sizeof(out_values[index].label),
                items[index].name);
        }
        index++;
    }

    g_variant_unref(data);
    *out_count = (unsigned short)((index < max_values) ? index : max_values);
    return SR_OK;
}

static int dsview_bridge_copy_channel_modes_for_current_operation(
    struct dsview_channel_mode *out_modes,
    int max_modes,
    unsigned short *out_count)
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
    if (status == SR_ERR_NA) {
        return SR_OK;
    }
    if (status != SR_OK) {
        return status;
    }
    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    items = (struct sr_list_item *)(uintptr_t)g_variant_get_uint64(data);
    while (items != NULL && items[index].id >= 0) {
        if (out_modes != NULL && index < max_modes) {
            int has_valid_channel_count = 0;
            int valid_channel_count = 0;

            out_modes[index].id = items[index].id;
            dsview_bridge_copy_string(
                out_modes[index].name,
                sizeof(out_modes[index].name),
                items[index].name);

            status = dsview_bridge_set_int16_config(SR_CONF_CHANNEL_MODE, items[index].id);
            if (status != SR_OK) {
                g_variant_unref(data);
                return status;
            }

            status = dsview_bridge_get_optional_int16_config(
                SR_CONF_VLD_CH_NUM,
                &has_valid_channel_count,
                &valid_channel_count);
            if (status != SR_OK) {
                g_variant_unref(data);
                return status;
            }

            out_modes[index].max_enabled_channels =
                has_valid_channel_count ? (unsigned short)valid_channel_count : 0;
        }
        index++;
    }

    g_variant_unref(data);
    *out_count = (unsigned short)((index < max_modes) ? index : max_modes);
    return SR_OK;
}

static int dsview_bridge_copy_validation_channel_modes_for_current_operation(
    struct dsview_validation_channel_mode *out_modes,
    int max_modes,
    unsigned short *out_count,
    unsigned short *out_total_channel_count,
    unsigned long long *out_hardware_sample_capacity)
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
    if (status == SR_ERR_NA) {
        return SR_OK;
    }
    if (status != SR_OK) {
        return status;
    }
    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    items = (struct sr_list_item *)(uintptr_t)g_variant_get_uint64(data);
    while (items != NULL && items[index].id >= 0) {
        if (out_modes != NULL && index < max_modes) {
            struct dsview_samplerate_list samplerates;
            int has_valid_channel_count = 0;
            int valid_channel_count = 0;

            memset(&samplerates, 0, sizeof(samplerates));
            memset(&out_modes[index], 0, sizeof(out_modes[index]));
            out_modes[index].code = items[index].id;
            dsview_bridge_copy_string(
                out_modes[index].label,
                sizeof(out_modes[index].label),
                items[index].name);

            status = dsview_bridge_set_int16_config(SR_CONF_CHANNEL_MODE, items[index].id);
            if (status != SR_OK) {
                g_variant_unref(data);
                return status;
            }

            status = dsview_bridge_get_optional_int16_config(
                SR_CONF_VLD_CH_NUM,
                &has_valid_channel_count,
                &valid_channel_count);
            if (status != SR_OK) {
                g_variant_unref(data);
                return status;
            }

            out_modes[index].max_enabled_channels =
                has_valid_channel_count ? (unsigned short)valid_channel_count : 0;

            status = dsview_bridge_ds_get_samplerates(&samplerates);
            if (status != SR_OK) {
                g_variant_unref(data);
                return status;
            }
            if (samplerates.count > DSVIEW_SAMPLERATE_CAPACITY) {
                samplerates.count = DSVIEW_SAMPLERATE_CAPACITY;
            }
            out_modes[index].samplerate_count = samplerates.count;
            memcpy(
                out_modes[index].samplerates,
                samplerates.values,
                samplerates.count * sizeof(samplerates.values[0]));

            if (out_total_channel_count != NULL && *out_total_channel_count == 0) {
                int total_channel_count = 0;

                status = dsview_bridge_ds_get_total_channel_count(&total_channel_count);
                if (status != SR_OK) {
                    g_variant_unref(data);
                    return status;
                }
                *out_total_channel_count = (unsigned short)total_channel_count;
            }

            if (out_hardware_sample_capacity != NULL && *out_hardware_sample_capacity == 0) {
                status = dsview_bridge_ds_get_hw_depth(out_hardware_sample_capacity);
                if (status != SR_OK) {
                    g_variant_unref(data);
                    return status;
                }
            }

        }
        index++;
    }

    g_variant_unref(data);
    *out_count = (unsigned short)((index < max_modes) ? index : max_modes);
    return SR_OK;
}

static int dsview_bridge_restore_device_modes(
    int has_operation_mode,
    int operation_mode,
    int has_channel_mode,
    int channel_mode)
{
    int status;

    if (has_operation_mode) {
        status = dsview_bridge_set_int16_config(SR_CONF_OPERATION_MODE, operation_mode);
        if (status != SR_OK) {
            return status;
        }
    }

    if (has_channel_mode) {
        status = dsview_bridge_set_int16_config(SR_CONF_CHANNEL_MODE, channel_mode);
        if (status != SR_OK) {
            return status;
        }
    }

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
    int enabled_channel_count = 0;
    size_t enabled_index;
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
    for (enabled_index = 0; enabled_index < G_N_ELEMENTS(g_enabled_channel_state); enabled_index++) {
        enabled_channel_count += g_enabled_channel_state[enabled_index] != 0;
    }
    if (enabled_channel_count <= 0) {
        enabled_channel_count = valid_channel_count;
    }

    if (sample_limit == 0 || enabled_channel_count <= 0) {
        return DSVIEW_EXPORT_ERR_NO_ENABLED_CHANNELS;
    }

    unitsize = (size_t)((enabled_channel_count + 7) / 8);
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
    g_recorded_stream.enabled_channel_count = (uint16_t)enabled_channel_count;
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
    retained.format = logic->format;
    retained.length = (size_t)logic->length;
    retained.unitsize = unitsize;
    retained.data_error = logic->data_error;
    retained.error_pattern = logic->error_pattern;
    retained.data = malloc(retained.length);
    if (retained.data == NULL) {
        return SR_ERR_MALLOC;
    }
    memcpy(retained.data, logic->data, retained.length);

    if (logic->format == LA_CROSS_DATA) {
        if (g_recorded_stream.enabled_channel_count == 0
            || ((size_t)logic->length % ((size_t)g_recorded_stream.enabled_channel_count * sizeof(uint64_t))) != 0) {
            free(retained.data);
            return DSVIEW_EXPORT_ERR_GENERIC;
        }
        packet_samples = ((unsigned long long)logic->length * 8ULL)
            / (unsigned long long)g_recorded_stream.enabled_channel_count;
    } else {
        packet_samples = logic->length / unitsize;
    }
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

int dsview_bridge_ds_get_current_operation_mode(int *value)
{
    return dsview_bridge_get_int16_config(SR_CONF_OPERATION_MODE, value);
}

int dsview_bridge_ds_get_current_stop_option(int *value)
{
    return dsview_bridge_get_int16_config(SR_CONF_BUFFER_OPTIONS, value);
}

int dsview_bridge_ds_get_current_filter(int *value)
{
    return dsview_bridge_get_int16_config(SR_CONF_FILTER, value);
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
    int has_original_channel_mode = 0;
    int original_channel_mode = 0;
    unsigned short copied_count = 0;
    int status;
    int restore_status = SR_OK;

    if (out_count == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    status = dsview_bridge_get_optional_int16_config(
        SR_CONF_CHANNEL_MODE,
        &has_original_channel_mode,
        &original_channel_mode);
    if (status != SR_OK) {
        return status;
    }

    status = dsview_bridge_copy_channel_modes_for_current_operation(
        out_modes,
        max_modes,
        &copied_count);

    if (has_original_channel_mode) {
        restore_status = dsview_bridge_set_int16_config(SR_CONF_CHANNEL_MODE, original_channel_mode);
    }

    if (status != SR_OK) {
        return (restore_status == SR_OK) ? status : restore_status;
    }

    *out_count = (int)copied_count;
    return restore_status;
}

int dsview_bridge_ds_get_device_options(struct dsview_device_options_snapshot *out_snapshot)
{
    int has_original_operation_mode = 0;
    int original_operation_mode = 0;
    int has_original_channel_mode = 0;
    int original_channel_mode = 0;
    int active_operation_mode = 0;
    int has_current_vth = 0;
    int has_current_threshold_code = 0;
    int index;
    int status;
    int restore_status = SR_OK;

    if (out_snapshot == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    memset(out_snapshot, 0, sizeof(*out_snapshot));
    dsview_bridge_copy_string(
        out_snapshot->threshold.kind,
        sizeof(out_snapshot->threshold.kind),
        "voltage-range");
    dsview_bridge_copy_string(
        out_snapshot->threshold.id,
        sizeof(out_snapshot->threshold.id),
        "threshold:vth-range");
    out_snapshot->threshold.min_volts = 0.0;
    out_snapshot->threshold.max_volts = 5.0;
    out_snapshot->threshold.step_volts = 0.1;

    status = dsview_bridge_get_optional_int16_config(
        SR_CONF_OPERATION_MODE,
        &out_snapshot->has_current_operation_mode,
        &out_snapshot->current_operation_mode_code);
    if (status != SR_OK) {
        return status;
    }
    status = dsview_bridge_get_optional_int16_config(
        SR_CONF_BUFFER_OPTIONS,
        &out_snapshot->has_current_stop_option,
        &out_snapshot->current_stop_option_code);
    if (status != SR_OK) {
        return status;
    }
    status = dsview_bridge_get_optional_int16_config(
        SR_CONF_FILTER,
        &out_snapshot->has_current_filter,
        &out_snapshot->current_filter_code);
    if (status != SR_OK) {
        return status;
    }
    status = dsview_bridge_get_optional_int16_config(
        SR_CONF_CHANNEL_MODE,
        &out_snapshot->has_current_channel_mode,
        &out_snapshot->current_channel_mode_code);
    if (status != SR_OK) {
        return status;
    }
    if (!out_snapshot->has_current_operation_mode || !out_snapshot->has_current_channel_mode) {
        return SR_ERR_NA;
    }
    status = dsview_bridge_get_optional_int16_config(
        SR_CONF_THRESHOLD,
        &has_current_threshold_code,
        &out_snapshot->threshold.current_legacy_code);
    if (status != SR_OK) {
        return status;
    }
    out_snapshot->threshold.has_current_legacy_code = has_current_threshold_code;
    status = dsview_bridge_get_optional_double_config(
        SR_CONF_VTH,
        &has_current_vth,
        &out_snapshot->threshold.current_volts);
    if (status != SR_OK) {
        return status;
    }
    out_snapshot->threshold.has_current_volts = has_current_vth;

    status = dsview_bridge_copy_option_values(
        SR_CONF_OPERATION_MODE,
        out_snapshot->operation_modes,
        DSVIEW_OPTION_VALUE_CAPACITY,
        &out_snapshot->operation_mode_count);
    if (status != SR_OK) {
        return status;
    }
    status = dsview_bridge_copy_option_values(
        SR_CONF_BUFFER_OPTIONS,
        out_snapshot->stop_options,
        DSVIEW_OPTION_VALUE_CAPACITY,
        &out_snapshot->stop_option_count);
    if (status != SR_OK) {
        return status;
    }
    status = dsview_bridge_copy_option_values(
        SR_CONF_FILTER,
        out_snapshot->filters,
        DSVIEW_OPTION_VALUE_CAPACITY,
        &out_snapshot->filter_count);
    if (status != SR_OK) {
        return status;
    }
    status = dsview_bridge_copy_option_values(
        SR_CONF_THRESHOLD,
        out_snapshot->threshold.legacy_options,
        DSVIEW_OPTION_VALUE_CAPACITY,
        &out_snapshot->threshold.legacy_option_count);
    if (status != SR_OK) {
        return status;
    }

    has_original_operation_mode = out_snapshot->has_current_operation_mode;
    original_operation_mode = out_snapshot->current_operation_mode_code;
    has_original_channel_mode = out_snapshot->has_current_channel_mode;
    original_channel_mode = out_snapshot->current_channel_mode_code;
    active_operation_mode = original_operation_mode;

    for (index = 0;
         index < out_snapshot->operation_mode_count &&
         index < DSVIEW_CHANNEL_MODE_GROUP_CAPACITY;
         index++) {
        struct dsview_channel_mode_group *group = &out_snapshot->channel_mode_groups[index];
        int operation_mode_code = out_snapshot->operation_modes[index].code;

        group->operation_mode_code = operation_mode_code;
        if (!has_original_operation_mode || active_operation_mode != operation_mode_code) {
            status = dsview_bridge_set_int16_config(SR_CONF_OPERATION_MODE, operation_mode_code);
            if (status != SR_OK) {
                goto restore;
            }
            active_operation_mode = operation_mode_code;
        }

        status = dsview_bridge_copy_channel_modes_for_current_operation(
            group->channel_modes,
            DSVIEW_CHANNEL_MODE_CAPACITY,
            &group->channel_mode_count);
        if (status != SR_OK) {
            goto restore;
        }

        out_snapshot->channel_mode_group_count++;
    }

restore:
    if (has_original_operation_mode || has_original_channel_mode) {
        restore_status = dsview_bridge_restore_device_modes(
            has_original_operation_mode,
            original_operation_mode,
            has_original_channel_mode,
            original_channel_mode);
    }

    if (status != SR_OK) {
        return (restore_status == SR_OK) ? status : restore_status;
    }

    return restore_status;
}

int dsview_bridge_ds_get_validation_capabilities(
    struct dsview_validation_capabilities_snapshot *out_snapshot)
{
    struct dsview_option_value operation_modes[DSVIEW_OPTION_VALUE_CAPACITY];
    unsigned short operation_mode_count = 0;
    int has_current_vth = 0;
    int has_current_threshold_code = 0;
    int index;
    int status = SR_OK;
    int restore_status = SR_OK;
    int has_original_operation_mode = 0;
    int original_operation_mode = 0;
    int has_original_channel_mode = 0;
    int original_channel_mode = 0;
    int active_operation_mode = 0;

    if (out_snapshot == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    memset(out_snapshot, 0, sizeof(*out_snapshot));
    memset(operation_modes, 0, sizeof(operation_modes));
    dsview_bridge_copy_string(
        out_snapshot->threshold.kind,
        sizeof(out_snapshot->threshold.kind),
        "voltage-range");
    dsview_bridge_copy_string(
        out_snapshot->threshold.id,
        sizeof(out_snapshot->threshold.id),
        "threshold:vth-range");
    out_snapshot->threshold.min_volts = 0.0;
    out_snapshot->threshold.max_volts = 5.0;
    out_snapshot->threshold.step_volts = 0.1;

    status = dsview_bridge_get_optional_int16_config(
        SR_CONF_OPERATION_MODE,
        &out_snapshot->has_current_operation_mode,
        &out_snapshot->current_operation_mode_code);
    if (status != SR_OK) {
        return status;
    }
    status = dsview_bridge_get_optional_int16_config(
        SR_CONF_BUFFER_OPTIONS,
        &out_snapshot->has_current_stop_option,
        &out_snapshot->current_stop_option_code);
    if (status != SR_OK) {
        return status;
    }
    status = dsview_bridge_get_optional_int16_config(
        SR_CONF_FILTER,
        &out_snapshot->has_current_filter,
        &out_snapshot->current_filter_code);
    if (status != SR_OK) {
        return status;
    }
    status = dsview_bridge_get_optional_int16_config(
        SR_CONF_CHANNEL_MODE,
        &out_snapshot->has_current_channel_mode,
        &out_snapshot->current_channel_mode_code);
    if (status != SR_OK) {
        return status;
    }
    if (!out_snapshot->has_current_operation_mode || !out_snapshot->has_current_channel_mode) {
        return SR_ERR_NA;
    }
    status = dsview_bridge_get_optional_int16_config(
        SR_CONF_THRESHOLD,
        &has_current_threshold_code,
        &out_snapshot->threshold.current_legacy_code);
    if (status != SR_OK) {
        return status;
    }
    out_snapshot->threshold.has_current_legacy_code = has_current_threshold_code;
    status = dsview_bridge_get_optional_double_config(
        SR_CONF_VTH,
        &has_current_vth,
        &out_snapshot->threshold.current_volts);
    if (status != SR_OK) {
        return status;
    }
    out_snapshot->threshold.has_current_volts = has_current_vth;

    status = dsview_bridge_copy_option_values(
        SR_CONF_FILTER,
        out_snapshot->filters,
        DSVIEW_OPTION_VALUE_CAPACITY,
        &out_snapshot->filter_count);
    if (status != SR_OK) {
        return status;
    }
    status = dsview_bridge_copy_option_values(
        SR_CONF_THRESHOLD,
        out_snapshot->threshold.legacy_options,
        DSVIEW_OPTION_VALUE_CAPACITY,
        &out_snapshot->threshold.legacy_option_count);
    if (status != SR_OK) {
        return status;
    }
    status = dsview_bridge_copy_option_values(
        SR_CONF_OPERATION_MODE,
        operation_modes,
        DSVIEW_OPTION_VALUE_CAPACITY,
        &operation_mode_count);
    if (status != SR_OK) {
        return status;
    }

    out_snapshot->operation_mode_count = operation_mode_count;
    has_original_operation_mode = out_snapshot->has_current_operation_mode;
    original_operation_mode = out_snapshot->current_operation_mode_code;
    has_original_channel_mode = out_snapshot->has_current_channel_mode;
    original_channel_mode = out_snapshot->current_channel_mode_code;
    active_operation_mode = original_operation_mode;

    for (index = 0;
         index < operation_mode_count && index < DSVIEW_CHANNEL_MODE_GROUP_CAPACITY;
         index++) {
        struct dsview_validation_operation_mode *operation_mode = &out_snapshot->operation_modes[index];
        int operation_mode_code = operation_modes[index].code;

        memset(operation_mode, 0, sizeof(*operation_mode));
        operation_mode->code = operation_mode_code;
        dsview_bridge_copy_string(
            operation_mode->label,
            sizeof(operation_mode->label),
            operation_modes[index].label);

        if (!has_original_operation_mode || active_operation_mode != operation_mode_code) {
            status = dsview_bridge_set_int16_config(SR_CONF_OPERATION_MODE, operation_mode_code);
            if (status != SR_OK) {
                goto restore;
            }
            active_operation_mode = operation_mode_code;
        }

        if (operation_mode_code == LO_OP_BUFFER) {
            status = dsview_bridge_copy_option_values(
                SR_CONF_BUFFER_OPTIONS,
                operation_mode->stop_options,
                DSVIEW_OPTION_VALUE_CAPACITY,
                &operation_mode->stop_option_count);
            if (status != SR_OK) {
                goto restore;
            }
        }

        status = dsview_bridge_copy_validation_channel_modes_for_current_operation(
            operation_mode->channel_modes,
            DSVIEW_CHANNEL_MODE_CAPACITY,
            &operation_mode->channel_mode_count,
            &out_snapshot->total_channel_count,
            &out_snapshot->hardware_sample_capacity);
        if (status != SR_OK) {
            goto restore;
        }
    }

restore:
    if (has_original_operation_mode || has_original_channel_mode) {
        restore_status = dsview_bridge_restore_device_modes(
            has_original_operation_mode,
            original_operation_mode,
            has_original_channel_mode,
            original_channel_mode);
    }

    if (status != SR_OK) {
        return (restore_status == SR_OK) ? status : restore_status;
    }

    return restore_status;
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

int dsview_bridge_ds_set_operation_mode(int value)
{
    return dsview_bridge_set_int16_config(SR_CONF_OPERATION_MODE, value);
}

int dsview_bridge_ds_set_stop_option(int value)
{
    return dsview_bridge_set_int16_config(SR_CONF_BUFFER_OPTIONS, value);
}

int dsview_bridge_ds_set_channel_mode(int value)
{
    return dsview_bridge_set_int16_config(SR_CONF_CHANNEL_MODE, value);
}

int dsview_bridge_ds_set_vth(double value)
{
    GVariant *data;

    if (g_bridge_api.ds_set_actived_device_config == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    data = g_variant_new_double(value);
    return g_bridge_api.ds_set_actived_device_config(NULL, NULL, SR_CONF_VTH, data);
}

int dsview_bridge_ds_set_filter(int value)
{
    return dsview_bridge_set_int16_config(SR_CONF_FILTER, value);
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
    int status;

    if (g_bridge_api.ds_enable_device_channel_index == NULL) {
        return DSVIEW_BRIDGE_ERR_NOT_LOADED;
    }

    status = g_bridge_api.ds_enable_device_channel_index(channel_index, enable ? TRUE : FALSE);
    if (status == SR_OK && channel_index >= 0
        && channel_index < (int)G_N_ELEMENTS(g_enabled_channel_state)) {
        g_enabled_channel_state[channel_index] = enable ? 1 : 0;
    }

    return status;
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
        logic.format = packet->format;
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

static int dsview_bridge_emit_cross_logic_packet(
    const struct sr_output *output,
    const struct dsview_retained_packet *packet,
    uint16_t enabled_channel_count,
    GString **assembled_output)
{
    struct sr_datafeed_packet replay_packet;
    struct sr_datafeed_logic logic;
    GString *chunk = NULL;
    uint8_t *expanded = NULL;
    size_t sample_blocks;
    size_t sample_count;
    uint16_t unitsize;
    size_t output_len;
    size_t block_index;
    size_t channel_index;
    int status;

    if (packet == NULL || packet->data == NULL || enabled_channel_count == 0) {
        return DSVIEW_EXPORT_ERR_GENERIC;
    }
    if ((packet->length % ((size_t)enabled_channel_count * sizeof(uint64_t))) != 0) {
        return DSVIEW_EXPORT_ERR_GENERIC;
    }

    sample_blocks = packet->length / ((size_t)enabled_channel_count * sizeof(uint64_t));
    sample_count = sample_blocks * 64U;
    unitsize = (uint16_t)((enabled_channel_count + 7U) / 8U);
    if (unitsize == 0) {
        unitsize = 1;
    }
    if (sample_count > (SIZE_MAX / unitsize)) {
        return DSVIEW_EXPORT_ERR_OVERFLOW;
    }

    output_len = sample_count * unitsize;
    expanded = calloc(output_len, 1);
    if (expanded == NULL) {
        return SR_ERR_MALLOC;
    }

    for (block_index = 0; block_index < sample_blocks; block_index++) {
        const uint8_t *block = packet->data
            + block_index * (size_t)enabled_channel_count * sizeof(uint64_t);

        for (channel_index = 0; channel_index < enabled_channel_count; channel_index++) {
            uint64_t bits = 0;
            size_t bit_index;

            memcpy(&bits, block + channel_index * sizeof(uint64_t), sizeof(bits));
            bits = GUINT64_FROM_LE(bits);

            for (bit_index = 0; bit_index < 64U; bit_index++) {
                if (((bits >> bit_index) & 1U) != 0) {
                    size_t sample_offset = (block_index * 64U + bit_index) * unitsize;
                    expanded[sample_offset + channel_index / 8U] |=
                        (uint8_t)(1U << (channel_index % 8U));
                }
            }
        }
    }

    memset(&logic, 0, sizeof(logic));
    logic.length = output_len;
    logic.format = LA_SPLIT_DATA;
    logic.unitsize = unitsize;
    logic.data_error = packet->data_error;
    logic.error_pattern = packet->error_pattern;
    logic.data = expanded;

    replay_packet.type = SR_DF_LOGIC;
    replay_packet.status = packet->status;
    replay_packet.payload = &logic;
    status = g_bridge_api.sr_output_send(output, &replay_packet, &chunk);
    if (status == SR_OK) {
        status = dsview_bridge_append_output_chunk(assembled_output, chunk);
    } else if (chunk != NULL) {
        g_string_free(chunk, TRUE);
    }

    free(expanded);
    return status;
}

static int dsview_bridge_export_stream(const struct dsview_vcd_export_request *request, const struct dsview_recorded_stream *stream, struct dsview_export_buffer *out_buffer)
{
    struct sr_dev_inst *sdi = NULL;
    const struct sr_output_module *module;
    const struct sr_output *output = NULL;
    GString *assembled_output = NULL;
    size_t index;
    unsigned long long exported_sample_count = 0;
    int status = SR_OK;
    int saw_meta = 0;
    int saw_end = 0;
    unsigned long long replay_samplerate_hz;
    uint16_t request_enabled_channel_count;

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
    if (request->enabled_channel_count == 0) {
        return DSVIEW_EXPORT_ERR_NO_ENABLED_CHANNELS;
    }

    replay_samplerate_hz = stream->samplerate_hz != 0 ? stream->samplerate_hz : request->samplerate_hz;
    request_enabled_channel_count = (uint16_t)request->enabled_channel_count;

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

        if (replay_packet.type == DSVIEW_EXPORT_PACKET_LOGIC
            && replay_packet.format == LA_CROSS_DATA) {
            exported_sample_count +=
                ((unsigned long long)replay_packet.length * 8ULL)
                / (unsigned long long)request_enabled_channel_count;
            status = dsview_bridge_emit_cross_logic_packet(
                output,
                &replay_packet,
                request_enabled_channel_count,
                &assembled_output);
        } else {
            if (replay_packet.type == DSVIEW_EXPORT_PACKET_LOGIC) {
                uint16_t logic_unitsize = replay_packet.unitsize != 0 ? replay_packet.unitsize : 1;
                exported_sample_count +=
                    (unsigned long long)(replay_packet.length / logic_unitsize);
            }
            status = dsview_bridge_emit_packet(output, &replay_packet, &assembled_output);
        }
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
    out_buffer->sample_count = exported_sample_count;
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
    packets[1].format = LA_SPLIT_DATA;
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
        logic_packet->format = LA_SPLIT_DATA;
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

int dsview_bridge_render_vcd_from_cross_logic_packets(
    const struct dsview_vcd_export_request *request,
    const uint8_t *sample_bytes,
    size_t sample_bytes_len,
    const size_t *logic_packet_lengths,
    size_t logic_packet_count,
    struct dsview_export_buffer *out_buffer)
{
    struct dsview_recorded_stream stream;
    struct dsview_retained_packet *packets = NULL;
    size_t packet_index;
    size_t offset = 0;
    size_t cross_unitsize;
    int status;

    if (request == NULL || sample_bytes == NULL || sample_bytes_len == 0 || logic_packet_lengths == NULL
        || logic_packet_count == 0 || out_buffer == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (request->enabled_channel_count == 0) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    cross_unitsize = request->enabled_channel_count * sizeof(uint64_t);
    if (cross_unitsize == 0 || (sample_bytes_len % cross_unitsize) != 0) {
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

        if (packet_length == 0 || (packet_length % cross_unitsize) != 0 || offset > sample_bytes_len
            || packet_length > sample_bytes_len - offset) {
            status = DSVIEW_BRIDGE_ERR_ARG;
            goto cleanup;
        }

        logic_packet->type = DSVIEW_EXPORT_PACKET_LOGIC;
        logic_packet->status = SR_PKT_OK;
        logic_packet->format = LA_CROSS_DATA;
        logic_packet->length = packet_length;
        logic_packet->unitsize = 1;
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
    stream.enabled_channel_count = (uint16_t)request->enabled_channel_count;

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

struct dsview_test_list_item {
    int id;
    char name[64];
};

struct dsview_test_channel_mode {
    int id;
    char name[64];
    unsigned short max_enabled_channels;
};

#define DSVIEW_TEST_APPLY_LOG_CAPACITY 256
#define DSVIEW_TEST_APPLY_FAILURE_CAPACITY 16
#define DSVIEW_TEST_CHANNEL_ENABLE_FAILURE_CAPACITY 16
#define DSVIEW_TEST_APPLY_KEY_CHANNEL_ENABLE (-70000)

struct dsview_test_mock_u64_state {
    int has_value;
    unsigned long long value;
    int status;
};

struct dsview_test_mock_int_state {
    int has_value;
    int value;
    int status;
};

struct dsview_test_mock_double_state {
    int has_value;
    double value;
    int status;
};

struct dsview_test_mock_list_state {
    int status;
    int count;
    struct sr_list_item list[DSVIEW_OPTION_VALUE_CAPACITY + 1];
    char labels[DSVIEW_OPTION_VALUE_CAPACITY][64];
};

struct dsview_test_mock_channel_group_state {
    int operation_mode_code;
    int status;
    int count;
    struct sr_list_item list[DSVIEW_CHANNEL_MODE_CAPACITY + 1];
    char labels[DSVIEW_CHANNEL_MODE_CAPACITY][64];
    unsigned short max_enabled_channels[DSVIEW_CHANNEL_MODE_CAPACITY];
    int samplerate_status[DSVIEW_CHANNEL_MODE_CAPACITY];
    unsigned int samplerate_count[DSVIEW_CHANNEL_MODE_CAPACITY];
    unsigned long long samplerates[DSVIEW_CHANNEL_MODE_CAPACITY][DSVIEW_SAMPLERATE_CAPACITY];
};

struct dsview_test_mock_apply_call {
    int key;
    long long value;
};

struct dsview_test_mock_apply_failure {
    int key;
    int status;
};

struct dsview_test_mock_channel_enable_failure {
    int channel_index;
    int enable;
    int status;
};

struct dsview_test_mock_state {
    struct dsview_test_mock_int_state current_operation_mode;
    struct dsview_test_mock_int_state current_stop_option;
    struct dsview_test_mock_int_state current_filter;
    struct dsview_test_mock_int_state current_channel_mode;
    struct dsview_test_mock_int_state current_threshold;
    struct dsview_test_mock_int_state current_valid_channel_count;
    struct dsview_test_mock_int_state current_total_channel_count;
    struct dsview_test_mock_double_state current_vth;
    struct dsview_test_mock_u64_state current_samplerate;
    struct dsview_test_mock_u64_state current_sample_limit;
    struct dsview_test_mock_u64_state current_hw_depth;
    struct dsview_test_mock_list_state operation_modes;
    struct dsview_test_mock_list_state stop_options;
    struct dsview_test_mock_list_state filters;
    struct dsview_test_mock_list_state legacy_thresholds;
    struct dsview_test_mock_channel_group_state channel_groups[DSVIEW_CHANNEL_MODE_GROUP_CAPACITY];
    struct dsview_test_mock_apply_call apply_log[DSVIEW_TEST_APPLY_LOG_CAPACITY];
    struct dsview_test_mock_apply_failure apply_failures[DSVIEW_TEST_APPLY_FAILURE_CAPACITY];
    struct dsview_test_mock_channel_enable_failure
        channel_enable_failures[DSVIEW_TEST_CHANNEL_ENABLE_FAILURE_CAPACITY];
    int channel_group_count;
    int apply_log_count;
    int apply_failure_count;
    int channel_enable_failure_count;
    int operation_mode_set_calls;
    int channel_mode_set_calls;
};

static struct dsview_test_mock_state g_test_mock_state;

static struct dsview_test_mock_int_state *dsview_test_mock_int_state_for_key(int key)
{
    switch (key) {
    case SR_CONF_OPERATION_MODE:
        return &g_test_mock_state.current_operation_mode;
    case SR_CONF_BUFFER_OPTIONS:
        return &g_test_mock_state.current_stop_option;
    case SR_CONF_FILTER:
        return &g_test_mock_state.current_filter;
    case SR_CONF_CHANNEL_MODE:
        return &g_test_mock_state.current_channel_mode;
    case SR_CONF_THRESHOLD:
        return &g_test_mock_state.current_threshold;
    case SR_CONF_VLD_CH_NUM:
        return &g_test_mock_state.current_valid_channel_count;
    case SR_CONF_TOTAL_CH_NUM:
        return &g_test_mock_state.current_total_channel_count;
    default:
        return NULL;
    }
}

static struct dsview_test_mock_u64_state *dsview_test_mock_u64_state_for_key(int key)
{
    switch (key) {
    case SR_CONF_SAMPLERATE:
        return &g_test_mock_state.current_samplerate;
    case SR_CONF_LIMIT_SAMPLES:
        return &g_test_mock_state.current_sample_limit;
    case SR_CONF_HW_DEPTH:
        return &g_test_mock_state.current_hw_depth;
    default:
        return NULL;
    }
}

static struct dsview_test_mock_list_state *dsview_test_mock_list_state_for_key(int key)
{
    switch (key) {
    case SR_CONF_OPERATION_MODE:
        return &g_test_mock_state.operation_modes;
    case SR_CONF_BUFFER_OPTIONS:
        return &g_test_mock_state.stop_options;
    case SR_CONF_FILTER:
        return &g_test_mock_state.filters;
    case SR_CONF_THRESHOLD:
        return &g_test_mock_state.legacy_thresholds;
    default:
        return NULL;
    }
}

static struct dsview_test_mock_channel_group_state *dsview_test_mock_find_channel_group(int operation_mode_code)
{
    int index;

    for (index = 0; index < g_test_mock_state.channel_group_count; index++) {
        if (g_test_mock_state.channel_groups[index].operation_mode_code == operation_mode_code) {
            return &g_test_mock_state.channel_groups[index];
        }
    }

    return NULL;
}

static const struct dsview_test_mock_channel_group_state *dsview_test_mock_current_channel_group(void)
{
    if (!g_test_mock_state.current_operation_mode.has_value) {
        return NULL;
    }

    return dsview_test_mock_find_channel_group(g_test_mock_state.current_operation_mode.value);
}

static void dsview_test_mock_record_apply_call(int key, long long value)
{
    if (g_test_mock_state.apply_log_count >= DSVIEW_TEST_APPLY_LOG_CAPACITY) {
        return;
    }

    g_test_mock_state.apply_log[g_test_mock_state.apply_log_count].key = key;
    g_test_mock_state.apply_log[g_test_mock_state.apply_log_count].value = value;
    g_test_mock_state.apply_log_count++;
}

static int dsview_test_mock_lookup_apply_failure(int key)
{
    int index;

    for (index = 0; index < g_test_mock_state.apply_failure_count; index++) {
        if (g_test_mock_state.apply_failures[index].key == key) {
            return g_test_mock_state.apply_failures[index].status;
        }
    }

    return SR_OK;
}

static int dsview_test_mock_lookup_channel_enable_failure(int channel_index, int enable)
{
    int index;

    for (index = 0; index < g_test_mock_state.channel_enable_failure_count; index++) {
        if (g_test_mock_state.channel_enable_failures[index].channel_index == channel_index
            && g_test_mock_state.channel_enable_failures[index].enable == enable) {
            return g_test_mock_state.channel_enable_failures[index].status;
        }
    }

    return SR_OK;
}

static int dsview_test_mock_valid_channels_for_mode(int channel_mode_code, unsigned short *out_value)
{
    const struct dsview_test_mock_channel_group_state *group = dsview_test_mock_current_channel_group();
    int index;

    if (out_value == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (group == NULL) {
        return SR_ERR_NA;
    }

    for (index = 0; index < group->count; index++) {
        if (group->list[index].id == channel_mode_code) {
            *out_value = group->max_enabled_channels[index];
            return SR_OK;
        }
    }

    return SR_ERR_ARG;
}

static int dsview_test_mock_samplerates_for_mode(
    int channel_mode_code,
    const unsigned long long **out_values,
    unsigned int *out_count)
{
    const struct dsview_test_mock_channel_group_state *group = dsview_test_mock_current_channel_group();
    int index;

    if (out_values == NULL || out_count == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }
    if (group == NULL) {
        return SR_ERR_NA;
    }

    for (index = 0; index < group->count; index++) {
        if (group->list[index].id == channel_mode_code) {
            *out_values = group->samplerates[index];
            *out_count = group->samplerate_count[index];
            return group->samplerate_status[index];
        }
    }

    return SR_ERR_ARG;
}

static int dsview_test_mock_get_config(
    const struct sr_channel *ch,
    const struct sr_channel_group *cg,
    int key,
    GVariant **data)
{
    struct dsview_test_mock_int_state *int_state;
    struct dsview_test_mock_u64_state *u64_state;

    (void)ch;
    (void)cg;

    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    if (key == SR_CONF_VTH) {
        if (g_test_mock_state.current_vth.status != SR_OK) {
            return g_test_mock_state.current_vth.status;
        }
        if (!g_test_mock_state.current_vth.has_value) {
            return SR_ERR_NA;
        }
        *data = g_variant_new_double(g_test_mock_state.current_vth.value);
        return SR_OK;
    }

    u64_state = dsview_test_mock_u64_state_for_key(key);
    if (u64_state != NULL) {
        if (u64_state->status != SR_OK) {
            return u64_state->status;
        }
        if (!u64_state->has_value) {
            return SR_ERR_NA;
        }
        *data = g_variant_new_uint64(u64_state->value);
        return SR_OK;
    }

    if (key == SR_CONF_VLD_CH_NUM) {
        unsigned short valid_channels = 0;
        int status = dsview_test_mock_valid_channels_for_mode(
            g_test_mock_state.current_channel_mode.value,
            &valid_channels);
        if (status == SR_OK) {
            *data = g_variant_new_int16((gint16)valid_channels);
            return SR_OK;
        }
        int_state = dsview_test_mock_int_state_for_key(key);
        if (int_state != NULL && int_state->status == SR_OK && int_state->has_value) {
            *data = g_variant_new_int16((gint16)int_state->value);
            return SR_OK;
        }
        return status;
    }

    int_state = dsview_test_mock_int_state_for_key(key);
    if (int_state == NULL) {
        return SR_ERR_NA;
    }
    if (int_state->status != SR_OK) {
        return int_state->status;
    }
    if (!int_state->has_value) {
        return SR_ERR_NA;
    }

    *data = g_variant_new_int16((gint16)int_state->value);
    return SR_OK;
}

static int dsview_test_mock_get_config_list(
    const struct sr_channel_group *cg,
    int key,
    GVariant **data)
{
    const struct dsview_test_mock_channel_group_state *group;
    struct dsview_test_mock_list_state *list_state;

    (void)cg;

    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    if (key == SR_CONF_CHANNEL_MODE) {
        group = dsview_test_mock_current_channel_group();
        if (group == NULL) {
            return SR_ERR_NA;
        }
        if (group->status != SR_OK) {
            return group->status;
        }
        *data = g_variant_new_uint64((guint64)(uintptr_t)group->list);
        return SR_OK;
    }

    if (key == SR_CONF_SAMPLERATE) {
        const unsigned long long *values = NULL;
        unsigned int count = 0;
        GVariantBuilder dict_builder;
        GVariantBuilder samplerate_builder;
        int index;
        int status = dsview_test_mock_samplerates_for_mode(
            g_test_mock_state.current_channel_mode.value,
            &values,
            &count);

        if (status != SR_OK) {
            return status;
        }

        g_variant_builder_init(&dict_builder, G_VARIANT_TYPE_VARDICT);
        g_variant_builder_init(&samplerate_builder, G_VARIANT_TYPE("at"));
        for (index = 0; index < (int)count; index++) {
            g_variant_builder_add(&samplerate_builder, "t", values[index]);
        }
        g_variant_builder_add(
            &dict_builder,
            "{sv}",
            "samplerates",
            g_variant_builder_end(&samplerate_builder));
        *data = g_variant_builder_end(&dict_builder);
        return SR_OK;
    }

    list_state = dsview_test_mock_list_state_for_key(key);
    if (list_state == NULL) {
        return SR_ERR_NA;
    }
    if (list_state->status != SR_OK) {
        return list_state->status;
    }

    *data = g_variant_new_uint64((guint64)(uintptr_t)list_state->list);
    return SR_OK;
}

static int dsview_test_mock_set_config(
    const struct sr_channel *ch,
    const struct sr_channel_group *cg,
    int key,
    GVariant *data)
{
    struct dsview_test_mock_channel_group_state *group;
    struct dsview_test_mock_u64_state *u64_state;
    double double_value;
    unsigned long long u64_value;
    int value;
    int index;
    int failure_status;

    (void)ch;
    (void)cg;

    if (data == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    if (key == SR_CONF_VTH) {
        double_value = g_variant_get_double(data);
        dsview_test_mock_record_apply_call(key, 0);
        failure_status = dsview_test_mock_lookup_apply_failure(key);
        if (failure_status != SR_OK) {
            return failure_status;
        }

        g_test_mock_state.current_vth.has_value = 1;
        g_test_mock_state.current_vth.value = double_value;
        g_test_mock_state.current_vth.status = SR_OK;
        return SR_OK;
    }

    u64_state = dsview_test_mock_u64_state_for_key(key);
    if (u64_state != NULL) {
        u64_value = g_variant_get_uint64(data);
        dsview_test_mock_record_apply_call(key, (long long)u64_value);
        failure_status = dsview_test_mock_lookup_apply_failure(key);
        if (failure_status != SR_OK) {
            return failure_status;
        }

        u64_state->has_value = 1;
        u64_state->value = u64_value;
        u64_state->status = SR_OK;
        return SR_OK;
    }

    value = g_variant_get_int16(data);
    dsview_test_mock_record_apply_call(key, value);
    failure_status = dsview_test_mock_lookup_apply_failure(key);
    if (failure_status != SR_OK) {
        return failure_status;
    }

    if (key == SR_CONF_OPERATION_MODE) {
        g_test_mock_state.operation_mode_set_calls++;
        group = dsview_test_mock_find_channel_group(value);
        if (group == NULL) {
            return SR_ERR_ARG;
        }

        g_test_mock_state.current_operation_mode.has_value = 1;
        g_test_mock_state.current_operation_mode.value = value;
        g_test_mock_state.current_operation_mode.status = SR_OK;
        if (group->count > 0) {
            g_test_mock_state.current_channel_mode.has_value = 1;
            g_test_mock_state.current_channel_mode.value = group->list[0].id;
            g_test_mock_state.current_channel_mode.status = SR_OK;
            g_test_mock_state.current_valid_channel_count.has_value = 1;
            g_test_mock_state.current_valid_channel_count.value = group->max_enabled_channels[0];
            g_test_mock_state.current_valid_channel_count.status = SR_OK;
        }
        return SR_OK;
    }

    if (key == SR_CONF_CHANNEL_MODE) {
        g_test_mock_state.channel_mode_set_calls++;
        group = dsview_test_mock_find_channel_group(g_test_mock_state.current_operation_mode.value);
        if (group == NULL) {
            return SR_ERR_ARG;
        }

        for (index = 0; index < group->count; index++) {
            if (group->list[index].id == value) {
                g_test_mock_state.current_channel_mode.has_value = 1;
                g_test_mock_state.current_channel_mode.value = value;
                g_test_mock_state.current_channel_mode.status = SR_OK;
                g_test_mock_state.current_valid_channel_count.has_value = 1;
                g_test_mock_state.current_valid_channel_count.value = group->max_enabled_channels[index];
                g_test_mock_state.current_valid_channel_count.status = SR_OK;
                return SR_OK;
            }
        }

        return SR_ERR_ARG;
    }

    if (key == SR_CONF_BUFFER_OPTIONS) {
        g_test_mock_state.current_stop_option.has_value = 1;
        g_test_mock_state.current_stop_option.value = value;
        g_test_mock_state.current_stop_option.status = SR_OK;
        return SR_OK;
    }

    if (key == SR_CONF_FILTER) {
        g_test_mock_state.current_filter.has_value = 1;
        g_test_mock_state.current_filter.value = value;
        g_test_mock_state.current_filter.status = SR_OK;
        return SR_OK;
    }

    return SR_ERR_NA;
}

static int dsview_test_mock_enable_channel(int channel_index, gboolean enable)
{
    int enabled = enable ? 1 : 0;
    int failure_status;

    dsview_test_mock_record_apply_call(
        DSVIEW_TEST_APPLY_KEY_CHANNEL_ENABLE,
        ((long long)channel_index << 1) | enabled);
    failure_status = dsview_test_mock_lookup_channel_enable_failure(channel_index, enabled);
    if (failure_status != SR_OK) {
        return failure_status;
    }

    return SR_OK;
}

void dsview_test_install_mock_option_api(void)
{
    g_bridge_api.ds_get_actived_device_config = dsview_test_mock_get_config;
    g_bridge_api.ds_get_actived_device_config_list = dsview_test_mock_get_config_list;
    g_bridge_api.ds_set_actived_device_config = dsview_test_mock_set_config;
    g_bridge_api.ds_enable_device_channel_index = dsview_test_mock_enable_channel;
}

void dsview_test_reset_mock_option_api(void)
{
    memset(&g_test_mock_state, 0, sizeof(g_test_mock_state));
    g_test_mock_state.current_operation_mode.status = SR_ERR_NA;
    g_test_mock_state.current_stop_option.status = SR_ERR_NA;
    g_test_mock_state.current_filter.status = SR_ERR_NA;
    g_test_mock_state.current_channel_mode.status = SR_ERR_NA;
    g_test_mock_state.current_threshold.status = SR_ERR_NA;
    g_test_mock_state.current_valid_channel_count.status = SR_ERR_NA;
    g_test_mock_state.current_total_channel_count.status = SR_ERR_NA;
    g_test_mock_state.current_vth.status = SR_ERR_NA;
    g_test_mock_state.current_samplerate.status = SR_ERR_NA;
    g_test_mock_state.current_sample_limit.status = SR_ERR_NA;
    g_test_mock_state.current_hw_depth.status = SR_ERR_NA;
    g_test_mock_state.operation_modes.status = SR_ERR_NA;
    g_test_mock_state.stop_options.status = SR_ERR_NA;
    g_test_mock_state.filters.status = SR_ERR_NA;
    g_test_mock_state.legacy_thresholds.status = SR_ERR_NA;
}

void dsview_test_mock_set_current_int(int key, int has_value, int value, int status)
{
    struct dsview_test_mock_int_state *state = dsview_test_mock_int_state_for_key(key);

    if (state == NULL) {
        return;
    }

    state->has_value = has_value != 0;
    state->value = value;
    state->status = status;
}

void dsview_test_mock_set_current_double(int key, int has_value, double value, int status)
{
    if (key != SR_CONF_VTH) {
        return;
    }

    g_test_mock_state.current_vth.has_value = has_value != 0;
    g_test_mock_state.current_vth.value = value;
    g_test_mock_state.current_vth.status = status;
}

void dsview_test_mock_set_current_u64(int key, int has_value, unsigned long long value, int status)
{
    struct dsview_test_mock_u64_state *state = dsview_test_mock_u64_state_for_key(key);

    if (state == NULL) {
        return;
    }

    state->has_value = has_value != 0;
    state->value = value;
    state->status = status;
}

void dsview_test_mock_set_list_items(
    int key,
    const struct dsview_test_list_item *items,
    int count,
    int status)
{
    struct dsview_test_mock_list_state *state = dsview_test_mock_list_state_for_key(key);
    int index;
    int capped_count;

    if (state == NULL) {
        return;
    }

    memset(state, 0, sizeof(*state));
    state->status = status;
    if (items == NULL || count <= 0) {
        state->list[0].id = -1;
        return;
    }

    capped_count = (count < DSVIEW_OPTION_VALUE_CAPACITY) ? count : DSVIEW_OPTION_VALUE_CAPACITY;
    state->count = capped_count;
    for (index = 0; index < capped_count; index++) {
        state->list[index].id = items[index].id;
        dsview_bridge_copy_string(state->labels[index], sizeof(state->labels[index]), items[index].name);
        state->list[index].name = state->labels[index];
    }
    state->list[capped_count].id = -1;
    state->list[capped_count].name = NULL;
}

void dsview_test_mock_set_channel_mode_group(
    int operation_mode_code,
    const struct dsview_test_channel_mode *items,
    int count,
    int status)
{
    struct dsview_test_mock_channel_group_state *group =
        dsview_test_mock_find_channel_group(operation_mode_code);
    int index;
    int capped_count;

    if (group == NULL) {
        if (g_test_mock_state.channel_group_count >= DSVIEW_CHANNEL_MODE_GROUP_CAPACITY) {
            return;
        }
        group = &g_test_mock_state.channel_groups[g_test_mock_state.channel_group_count++];
    }

    memset(group, 0, sizeof(*group));
    group->operation_mode_code = operation_mode_code;
    group->status = status;
    if (items == NULL || count <= 0) {
        group->list[0].id = -1;
        return;
    }

    capped_count = (count < DSVIEW_CHANNEL_MODE_CAPACITY) ? count : DSVIEW_CHANNEL_MODE_CAPACITY;
    group->count = capped_count;
    for (index = 0; index < capped_count; index++) {
        group->list[index].id = items[index].id;
        dsview_bridge_copy_string(group->labels[index], sizeof(group->labels[index]), items[index].name);
        group->list[index].name = group->labels[index];
        group->max_enabled_channels[index] = items[index].max_enabled_channels;
    }
    group->list[capped_count].id = -1;
    group->list[capped_count].name = NULL;
}

void dsview_test_mock_set_channel_mode_samplerates(
    int operation_mode_code,
    int channel_mode_code,
    const unsigned long long *values,
    int count,
    int status)
{
    struct dsview_test_mock_channel_group_state *group =
        dsview_test_mock_find_channel_group(operation_mode_code);
    int index;
    int capped_count;

    if (group == NULL) {
        return;
    }

    for (index = 0; index < group->count; index++) {
        if (group->list[index].id == channel_mode_code) {
            group->samplerate_status[index] = status;
            if (values == NULL || count <= 0) {
                group->samplerate_count[index] = 0;
                return;
            }

            capped_count = (count < DSVIEW_SAMPLERATE_CAPACITY)
                ? count
                : DSVIEW_SAMPLERATE_CAPACITY;
            group->samplerate_count[index] = (unsigned int)capped_count;
            memcpy(
                group->samplerates[index],
                values,
                capped_count * sizeof(values[0]));
            return;
        }
    }
}

int dsview_test_mock_get_current_int(int key, int *out_has_value, int *out_value)
{
    const struct dsview_test_mock_int_state *state = dsview_test_mock_int_state_for_key(key);

    if (state == NULL || out_has_value == NULL || out_value == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    *out_has_value = state->has_value;
    *out_value = state->value;
    return state->status;
}

int dsview_test_mock_get_current_double(int key, int *out_has_value, double *out_value)
{
    if (key != SR_CONF_VTH || out_has_value == NULL || out_value == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    *out_has_value = g_test_mock_state.current_vth.has_value;
    *out_value = g_test_mock_state.current_vth.value;
    return g_test_mock_state.current_vth.status;
}

int dsview_test_mock_get_current_u64(int key, int *out_has_value, unsigned long long *out_value)
{
    const struct dsview_test_mock_u64_state *state = dsview_test_mock_u64_state_for_key(key);

    if (state == NULL || out_has_value == NULL || out_value == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    *out_has_value = state->has_value;
    *out_value = state->value;
    return state->status;
}

void dsview_test_mock_set_apply_failure(int key, int status)
{
    int index;

    for (index = 0; index < g_test_mock_state.apply_failure_count; index++) {
        if (g_test_mock_state.apply_failures[index].key == key) {
            g_test_mock_state.apply_failures[index].status = status;
            return;
        }
    }

    if (g_test_mock_state.apply_failure_count >= DSVIEW_TEST_APPLY_FAILURE_CAPACITY) {
        return;
    }

    g_test_mock_state.apply_failures[g_test_mock_state.apply_failure_count].key = key;
    g_test_mock_state.apply_failures[g_test_mock_state.apply_failure_count].status = status;
    g_test_mock_state.apply_failure_count++;
}

void dsview_test_mock_set_channel_enable_failure(int channel_index, int enable, int status)
{
    int index;

    for (index = 0; index < g_test_mock_state.channel_enable_failure_count; index++) {
        if (g_test_mock_state.channel_enable_failures[index].channel_index == channel_index
            && g_test_mock_state.channel_enable_failures[index].enable == enable) {
            g_test_mock_state.channel_enable_failures[index].status = status;
            return;
        }
    }

    if (g_test_mock_state.channel_enable_failure_count
        >= DSVIEW_TEST_CHANNEL_ENABLE_FAILURE_CAPACITY) {
        return;
    }

    g_test_mock_state.channel_enable_failures[g_test_mock_state.channel_enable_failure_count]
        .channel_index = channel_index;
    g_test_mock_state.channel_enable_failures[g_test_mock_state.channel_enable_failure_count]
        .enable = enable;
    g_test_mock_state.channel_enable_failures[g_test_mock_state.channel_enable_failure_count]
        .status = status;
    g_test_mock_state.channel_enable_failure_count++;
}

void dsview_test_mock_reset_apply_log(void)
{
    g_test_mock_state.apply_log_count = 0;
    g_test_mock_state.apply_failure_count = 0;
    g_test_mock_state.channel_enable_failure_count = 0;
}

int dsview_test_mock_get_apply_call_count(void)
{
    return g_test_mock_state.apply_log_count;
}

int dsview_test_mock_get_apply_call(int index, int *out_key, long long *out_value)
{
    if (index < 0 || index >= g_test_mock_state.apply_log_count || out_key == NULL || out_value == NULL) {
        return DSVIEW_BRIDGE_ERR_ARG;
    }

    *out_key = g_test_mock_state.apply_log[index].key;
    *out_value = g_test_mock_state.apply_log[index].value;
    return SR_OK;
}

int dsview_test_mock_get_set_call_count(int key)
{
    if (key == SR_CONF_OPERATION_MODE) {
        return g_test_mock_state.operation_mode_set_calls;
    }
    if (key == SR_CONF_CHANNEL_MODE) {
        return g_test_mock_state.channel_mode_set_calls;
    }

    return 0;
}
