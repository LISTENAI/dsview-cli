#include "libsigrok.h"

SR_API struct sr_input_format **sr_input_list(void)
{
    static struct sr_input_format *input_list[] = {
        NULL,
    };

    return input_list;
}
