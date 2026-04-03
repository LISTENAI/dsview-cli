#ifndef DSVIEW_SYS_WRAPPER_H
#define DSVIEW_SYS_WRAPPER_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t ds_device_handle;

#define NULL_HANDLE ((ds_device_handle)0)

struct ds_device_base_info {
    ds_device_handle handle;
    char name[150];
};

enum dsview_bridge_status {
    DSVIEW_BRIDGE_OK = 0,
    DSVIEW_BRIDGE_ERR_ARG = -1,
    DSVIEW_BRIDGE_ERR_NOT_LOADED = -2,
    DSVIEW_BRIDGE_ERR_DLOPEN = -3,
    DSVIEW_BRIDGE_ERR_DLSYM = -4,
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

#ifdef __cplusplus
}
#endif

#endif
