#ifndef DSVIEW_SYS_WRAPPER_H
#define DSVIEW_SYS_WRAPPER_H

#include <stddef.h>
#include <stdint.h>

#include "libsigrok.h"
#include "libsigrokdecode.h"

#ifdef __cplusplus
extern "C" {
#endif

enum dsview_acquisition_terminal_event {
    DSVIEW_ACQ_TERMINAL_NONE = 0,
    DSVIEW_ACQ_TERMINAL_NORMAL_END = 1,
    DSVIEW_ACQ_TERMINAL_END_BY_DETACHED = 2,
    DSVIEW_ACQ_TERMINAL_END_BY_ERROR = 3,
};

struct dsview_bridge_acquisition_summary {
    int callback_registration_active;
    int start_status;
    int saw_collect_task_start;
    int saw_device_running;
    int saw_device_stopped;
    int saw_terminal_normal_end;
    int saw_terminal_end_by_detached;
    int saw_terminal_end_by_error;
    int terminal_event;
    int saw_logic_packet;
    int saw_end_packet;
    int end_packet_status;
    int saw_end_packet_ok;
    int saw_data_error_packet;
    int last_error;
    int is_collecting;
};

struct dsview_vcd_export_request {
    unsigned long long samplerate_hz;
    const uint16_t *enabled_channels;
    size_t enabled_channel_count;
};

struct dsview_export_buffer {
    uint8_t *data;
    size_t len;
    unsigned long long sample_count;
    size_t packet_count;
};

struct dsview_stream_export_facts {
    unsigned long long sample_count;
    size_t packet_count;
    unsigned long long output_bytes;
};

enum dsview_bridge_status {
    DSVIEW_BRIDGE_OK = 0,
    DSVIEW_BRIDGE_ERR_ARG = -1,
    DSVIEW_BRIDGE_ERR_NOT_LOADED = -2,
    DSVIEW_BRIDGE_ERR_DLOPEN = -3,
    DSVIEW_BRIDGE_ERR_DLSYM = -4,
};

enum dsview_decode_status {
    DSVIEW_DECODE_OK = 0,
    DSVIEW_DECODE_ERR_ARG = -20,
    DSVIEW_DECODE_ERR_NOT_LOADED = -21,
    DSVIEW_DECODE_ERR_DECODER_DIR = -22,
    DSVIEW_DECODE_ERR_PYTHON = -23,
    DSVIEW_DECODE_ERR_DECODER_LOAD = -24,
    DSVIEW_DECODE_ERR_UNKNOWN_DECODER = -25,
    DSVIEW_DECODE_ERR_UPSTREAM = -26,
    DSVIEW_DECODE_ERR_MALLOC = -27,
    DSVIEW_DECODE_ERR_INPUT_SHAPE = -28,
    DSVIEW_DECODE_ERR_SESSION = -29,
};

struct dsview_channel_mode {
    int id;
    char name[64];
    unsigned short max_enabled_channels;
};

#define DSVIEW_OPTION_VALUE_CAPACITY 16
#define DSVIEW_CHANNEL_MODE_GROUP_CAPACITY 8
#define DSVIEW_CHANNEL_MODE_CAPACITY 16
#define DSVIEW_SAMPLERATE_CAPACITY 64

struct dsview_option_value {
    int code;
    char label[64];
};

struct dsview_channel_mode_group {
    int operation_mode_code;
    unsigned short channel_mode_count;
    struct dsview_channel_mode channel_modes[DSVIEW_CHANNEL_MODE_CAPACITY];
};

struct dsview_validation_channel_mode {
    int code;
    char label[64];
    unsigned short max_enabled_channels;
    unsigned int samplerate_count;
    unsigned long long samplerates[DSVIEW_SAMPLERATE_CAPACITY];
};

struct dsview_validation_operation_mode {
    int code;
    char label[64];
    unsigned short stop_option_count;
    struct dsview_option_value stop_options[DSVIEW_OPTION_VALUE_CAPACITY];
    unsigned short channel_mode_count;
    struct dsview_validation_channel_mode channel_modes[DSVIEW_CHANNEL_MODE_CAPACITY];
};

struct dsview_threshold_range {
    char kind[32];
    char id[64];
    int has_current_volts;
    double current_volts;
    double min_volts;
    double max_volts;
    double step_volts;
    int has_current_legacy_code;
    int current_legacy_code;
    unsigned short legacy_option_count;
    struct dsview_option_value legacy_options[DSVIEW_OPTION_VALUE_CAPACITY];
};

struct dsview_device_options_snapshot {
    int has_current_operation_mode;
    int current_operation_mode_code;
    unsigned short operation_mode_count;
    struct dsview_option_value operation_modes[DSVIEW_OPTION_VALUE_CAPACITY];
    int has_current_stop_option;
    int current_stop_option_code;
    unsigned short stop_option_count;
    struct dsview_option_value stop_options[DSVIEW_OPTION_VALUE_CAPACITY];
    int has_current_filter;
    int current_filter_code;
    unsigned short filter_count;
    struct dsview_option_value filters[DSVIEW_OPTION_VALUE_CAPACITY];
    int has_current_channel_mode;
    int current_channel_mode_code;
    unsigned short channel_mode_group_count;
    struct dsview_channel_mode_group channel_mode_groups[DSVIEW_CHANNEL_MODE_GROUP_CAPACITY];
    struct dsview_threshold_range threshold;
};

struct dsview_validation_capabilities_snapshot {
    int has_current_operation_mode;
    int current_operation_mode_code;
    int has_current_stop_option;
    int current_stop_option_code;
    int has_current_filter;
    int current_filter_code;
    int has_current_channel_mode;
    int current_channel_mode_code;
    unsigned short total_channel_count;
    unsigned long long hardware_sample_capacity;
    unsigned short filter_count;
    struct dsview_option_value filters[DSVIEW_OPTION_VALUE_CAPACITY];
    struct dsview_threshold_range threshold;
    unsigned short operation_mode_count;
    struct dsview_validation_operation_mode operation_modes[DSVIEW_CHANNEL_MODE_GROUP_CAPACITY];
};

struct dsview_samplerate_list {
    unsigned int count;
    unsigned long long values[64];
};

struct dsview_decode_channel {
    char *id;
    char *name;
    char *desc;
    int order;
    int type;
    char *idn;
};

enum dsview_decode_option_value_kind {
    DSVIEW_DECODE_OPTION_VALUE_KIND_UNKNOWN = 0,
    DSVIEW_DECODE_OPTION_VALUE_KIND_STRING = 1,
    DSVIEW_DECODE_OPTION_VALUE_KIND_INTEGER = 2,
    DSVIEW_DECODE_OPTION_VALUE_KIND_FLOAT = 3,
};

enum dsview_decode_logic_format {
    DSVIEW_DECODE_LOGIC_FORMAT_SPLIT = 1,
    DSVIEW_DECODE_LOGIC_FORMAT_CROSS = 2,
};

struct dsview_decode_option_value {
    int kind;
    const char *string_value;
    long long integer_value;
    double float_value;
};

struct dsview_decode_option_entry {
    const char *option_id;
    struct dsview_decode_option_value value;
};

struct dsview_decode_channel_binding {
    const char *channel_id;
    unsigned int channel_index;
};

struct dsview_decode_instance_spec {
    const char *decoder_id;
    const struct dsview_decode_channel_binding *channel_bindings;
    size_t channel_binding_count;
    const struct dsview_decode_option_entry *options;
    size_t option_count;
};

struct dsview_decode_logic_chunk {
    int format;
    uint16_t unitsize;
    uint16_t channel_count;
    uint64_t abs_start_sample;
    uint64_t abs_end_sample;
    const uint8_t *sample_bytes;
    size_t sample_bytes_len;
};

struct dsview_decode_execution_session;

struct dsview_decode_option {
    char *id;
    char *idn;
    char *desc;
    int value_kind;
    char *default_value;
    char **values;
    size_t value_count;
};

struct dsview_decode_annotation {
    char *id;
    char *label;
    char *description;
    int type;
};

struct dsview_decode_annotation_row {
    char *id;
    char *desc;
    size_t *annotation_classes;
    size_t annotation_class_count;
};

struct dsview_decode_captured_annotation {
    char *decoder_id;
    uint64_t start_sample;
    uint64_t end_sample;
    int ann_class;
    int ann_type;
    char **texts;
    size_t text_count;
};

struct dsview_decode_list_entry {
    char *id;
    char *name;
    char *longname;
    char *desc;
    char *license;
};

struct dsview_decode_metadata {
    char *id;
    char *name;
    char *longname;
    char *desc;
    char *license;
    char **inputs;
    size_t input_count;
    char **outputs;
    size_t output_count;
    char **tags;
    size_t tag_count;
    struct dsview_decode_channel *required_channels;
    size_t required_channel_count;
    struct dsview_decode_channel *optional_channels;
    size_t optional_channel_count;
    struct dsview_decode_option *options;
    size_t option_count;
    struct dsview_decode_annotation *annotations;
    size_t annotation_count;
    struct dsview_decode_annotation_row *annotation_rows;
    size_t annotation_row_count;
};

int dsview_bridge_load_library(const char *path);
void dsview_bridge_unload_library(void);
int dsview_bridge_is_loaded(void);
const char *dsview_bridge_last_loader_error(void);

int dsview_bridge_ds_lib_init(void);
int dsview_bridge_ds_lib_exit(void);
void dsview_bridge_ds_set_firmware_resource_dir(const char *dir);
int dsview_bridge_ds_get_device_list(struct ds_device_base_info **out_list, int *out_count);
void dsview_bridge_free_device_list(struct ds_device_base_info *list);
int dsview_bridge_ds_active_device(ds_device_handle handle);
int dsview_bridge_ds_release_actived_device(void);
int dsview_bridge_ds_get_last_error(void);
int dsview_bridge_ds_get_actived_device_init_status(int *status);
int dsview_bridge_ds_get_current_samplerate(unsigned long long *value);
int dsview_bridge_ds_get_current_sample_limit(unsigned long long *value);
int dsview_bridge_ds_get_total_channel_count(int *value);
int dsview_bridge_ds_get_valid_channel_count(int *value);
int dsview_bridge_ds_get_current_channel_mode(int *value);
int dsview_bridge_ds_get_hw_depth(unsigned long long *value);
int dsview_bridge_ds_get_vth(double *value);
int dsview_bridge_ds_get_samplerates(struct dsview_samplerate_list *out_list);
int dsview_bridge_ds_get_channel_modes(struct dsview_channel_mode *out_modes, int max_modes, int *out_count);
int dsview_bridge_ds_get_device_options(struct dsview_device_options_snapshot *out_snapshot);
int dsview_bridge_ds_get_validation_capabilities(struct dsview_validation_capabilities_snapshot *out_snapshot);
int dsview_bridge_ds_set_samplerate(unsigned long long value);
int dsview_bridge_ds_set_sample_limit(unsigned long long value);
int dsview_bridge_ds_enable_channel(int channel_index, int enable);
int dsview_bridge_ds_register_acquisition_callbacks(void);
int dsview_bridge_ds_clear_acquisition_callbacks(void);
int dsview_bridge_ds_start_collect(void);
int dsview_bridge_ds_stop_collect(void);
int dsview_bridge_ds_is_collecting(int *value);
int dsview_bridge_ds_reset_acquisition_summary(void);
int dsview_bridge_ds_get_acquisition_summary(struct dsview_bridge_acquisition_summary *out_summary);
int dsview_bridge_ds_export_recorded_vcd(
    const struct dsview_vcd_export_request *request,
    struct dsview_export_buffer *out_buffer);
int dsview_bridge_ds_begin_streaming_vcd(
    const struct dsview_vcd_export_request *request,
    const char *path);
int dsview_bridge_ds_finish_streaming_vcd(struct dsview_stream_export_facts *out_facts);
void dsview_bridge_ds_abort_streaming_vcd(void);
int dsview_bridge_render_vcd_from_samples(
    const struct dsview_vcd_export_request *request,
    const uint8_t *sample_bytes,
    size_t sample_bytes_len,
    uint16_t unitsize,
    struct dsview_export_buffer *out_buffer);
int dsview_bridge_render_vcd_from_logic_packets(
    const struct dsview_vcd_export_request *request,
    const uint8_t *sample_bytes,
    size_t sample_bytes_len,
    const size_t *logic_packet_lengths,
    size_t logic_packet_count,
    uint16_t unitsize,
    struct dsview_export_buffer *out_buffer);
int dsview_bridge_render_vcd_from_cross_logic_packets(
    const struct dsview_vcd_export_request *request,
    const uint8_t *sample_bytes,
    size_t sample_bytes_len,
    const size_t *logic_packet_lengths,
    size_t logic_packet_count,
    struct dsview_export_buffer *out_buffer);
void dsview_bridge_free_export_buffer(struct dsview_export_buffer *buffer);

int dsview_decode_runtime_load(const char *path);
void dsview_decode_runtime_unload(void);
int dsview_decode_runtime_init(const char *decoder_dir);
int dsview_decode_runtime_exit(void);
const char *dsview_decode_last_loader_error(void);
const char *dsview_decode_last_error(void);
const char *dsview_decode_last_error_name(void);
int dsview_decode_list(struct dsview_decode_list_entry **out_list, size_t *out_count);
void dsview_decode_free_list(struct dsview_decode_list_entry *list, size_t count);
int dsview_decode_inspect(const char *decoder_id, struct dsview_decode_metadata *out_metadata);
void dsview_decode_free_metadata(struct dsview_decode_metadata *metadata);
int dsview_decode_session_new(struct dsview_decode_execution_session **out_session);
int dsview_decode_session_set_samplerate(
    struct dsview_decode_execution_session *session,
    unsigned long long samplerate_hz);
int dsview_decode_session_build_linear_stack(
    struct dsview_decode_execution_session *session,
    const struct dsview_decode_instance_spec *root,
    const struct dsview_decode_instance_spec *stack,
    size_t stack_count);
int dsview_decode_session_start(struct dsview_decode_execution_session *session);
int dsview_decode_session_send_logic_chunk(
    struct dsview_decode_execution_session *session,
    const struct dsview_decode_logic_chunk *chunk);
int dsview_decode_session_end(struct dsview_decode_execution_session *session);
int dsview_decode_session_take_captured_annotations(
    struct dsview_decode_execution_session *session,
    struct dsview_decode_captured_annotation **out_annotations,
    size_t *out_count);
void dsview_decode_free_captured_annotations(
    struct dsview_decode_captured_annotation *annotations,
    size_t count);
void dsview_decode_session_destroy(struct dsview_decode_execution_session *session);

#ifdef __cplusplus
}
#endif

#endif
