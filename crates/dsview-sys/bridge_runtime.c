#include "wrapper.h"

#include <dlfcn.h>
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

struct dsview_bridge_api {
    void *library_handle;
    ds_lib_init_fn ds_lib_init;
    ds_lib_exit_fn ds_lib_exit;
    ds_set_firmware_resource_dir_fn ds_set_firmware_resource_dir;
    ds_get_device_list_fn ds_get_device_list;
    ds_active_device_fn ds_active_device;
    ds_release_actived_device_fn ds_release_actived_device;
    ds_get_last_error_fn ds_get_last_error;
    ds_get_actived_device_init_status_fn ds_get_actived_device_init_status;
    char last_error[512];
};

static struct dsview_bridge_api g_bridge_api;

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

    dsview_bridge_set_error_from_text(NULL);
    return DSVIEW_BRIDGE_OK;
}

void dsview_bridge_unload_library(void)
{
    if (g_bridge_api.library_handle != NULL) {
        dlclose(g_bridge_api.library_handle);
    }

    memset(&g_bridge_api, 0, sizeof(g_bridge_api));
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
