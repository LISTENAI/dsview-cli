#ifndef DSVIEW_SYS_WRAPPER_H
#define DSVIEW_SYS_WRAPPER_H

#include <stddef.h>
#include <stdint.h>

#include "libsigrok.h"

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

enum dsview_bridge_status {
    DSVIEW_BRIDGE_OK = 0,
    DSVIEW_BRIDGE_ERR_ARG = -1,
    DSVIEW_BRIDGE_ERR_NOT_LOADED = -2,
    DSVIEW_BRIDGE_ERR_DLOPEN = -3,
    DSVIEW_BRIDGE_ERR_DLSYM = -4,
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
void dsview_bridge_free_export_buffer(struct dsview_export_buffer *buffer);

#ifdef __cplusplus
}
#endif

#endif
