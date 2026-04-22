#include "libsigrok-internal.h"

SR_PRIV int sr_new_virtual_device(const char *filename, struct sr_dev_inst **out_di)
{
    (void)filename;
    if (out_di != NULL) {
        *out_di = NULL;
    }

    return SR_ERR_NA;
}
