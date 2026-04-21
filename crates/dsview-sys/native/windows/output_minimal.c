#include "libsigrok-internal.h"
#include <glib.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#undef LOG_PREFIX
#define LOG_PREFIX "vcd: "

struct minimal_vcd_context {
    int num_enabled_channels;
    int *channel_index;
    uint8_t *prevsample;
    gboolean header_done;
    uint64_t samplerate;
    uint64_t samplecount;
    int period;
};

static const char *const output_vcd_exts[] = {"vcd", NULL};

static int minimal_vcd_init(struct sr_output *o, GHashTable *options)
{
    struct minimal_vcd_context *ctx;
    struct sr_channel *channel;
    GSList *node;
    int index;

    (void)options;

    ctx = g_malloc0(sizeof(*ctx));
    if (ctx == NULL) {
        sr_err("%s,ERROR:failed to alloc memory.", __func__);
        return SR_ERR_MALLOC;
    }

    for (node = o->sdi->channels; node != NULL; node = node->next) {
        channel = node->data;
        if (channel->type == SR_CHANNEL_LOGIC && channel->enabled) {
            ctx->num_enabled_channels++;
        }
    }

    if (ctx->num_enabled_channels <= 0 || ctx->num_enabled_channels > 94) {
        g_free(ctx);
        return SR_ERR;
    }

    ctx->channel_index = g_new(int, ctx->num_enabled_channels);
    if (ctx->channel_index == NULL) {
        g_free(ctx);
        return SR_ERR_MALLOC;
    }

    index = 0;
    for (node = o->sdi->channels; node != NULL; node = node->next) {
        channel = node->data;
        if (channel->type == SR_CHANNEL_LOGIC && channel->enabled) {
            ctx->channel_index[index++] = channel->index;
        }
    }

    o->priv = ctx;
    return SR_OK;
}

static GString *minimal_vcd_header(const struct sr_output *o)
{
    struct minimal_vcd_context *ctx = o->priv;
    struct sr_channel *channel;
    GSList *node;
    GString *header;
    char timestamp[64];
    time_t now;
    struct tm local_time;
    int signal_index;
    int total_channels;

    header = g_string_sized_new(512);
    total_channels = g_slist_length(o->sdi->channels);

    now = time(NULL);
#if defined(_WIN32)
    localtime_s(&local_time, &now);
#else
    localtime_r(&now, &local_time);
#endif
    strftime(timestamp, sizeof(timestamp), "%c", &local_time);

    g_string_printf(header, "$date %s $end\n", timestamp);
    g_string_append(header, "$version dsview_runtime $end\n");
    g_string_append_printf(
        header,
        "$comment\n  Acquisition with %d/%d channels\n$end\n",
        ctx->num_enabled_channels,
        total_channels
    );

    if (ctx->samplerate == 0) {
        ctx->samplerate = SR_MHZ(1);
    }

    if (ctx->samplerate > SR_MHZ(1)) {
        ctx->period = SR_GHZ(1);
    } else if (ctx->samplerate > SR_KHZ(1)) {
        ctx->period = SR_MHZ(1);
    } else {
        ctx->period = SR_KHZ(1);
    }

    g_string_append(header, "$timescale 1 ns $end\n");
    g_string_append(header, "$scope module dsview_runtime $end\n");

    signal_index = 0;
    for (node = o->sdi->channels; node != NULL; node = node->next) {
        channel = node->data;
        if (channel->type != SR_CHANNEL_LOGIC || !channel->enabled) {
            continue;
        }

        g_string_append_printf(
            header,
            "$var wire 1 %c %s $end\n",
            (char)('!' + signal_index),
            channel->name
        );
        signal_index++;
    }

    g_string_append(header, "$upscope $end\n$enddefinitions $end\n");
    return header;
}

static int minimal_vcd_receive(
    const struct sr_output *o,
    const struct sr_datafeed_packet *packet,
    GString **out
)
{
    const struct sr_datafeed_logic *logic;
    const struct sr_datafeed_meta *meta;
    const struct sr_config *config;
    struct minimal_vcd_context *ctx;
    GSList *node;
    const uint8_t *sample;
    unsigned int offset;
    int bit_index;
    int channel_index;
    int current_bit;
    int previous_bit;
    gboolean timestamp_written;

    *out = NULL;
    ctx = o->priv;
    if (ctx == NULL) {
        return SR_ERR_BUG;
    }

    switch (packet->type) {
    case SR_DF_META:
        meta = packet->payload;
        for (node = meta->config; node != NULL; node = node->next) {
            config = node->data;
            if (config->key == SR_CONF_SAMPLERATE) {
                ctx->samplerate = g_variant_get_uint64(config->data);
            }
        }
        return SR_OK;
    case SR_DF_LOGIC:
        logic = packet->payload;
        if (!ctx->header_done) {
            *out = minimal_vcd_header(o);
            ctx->header_done = TRUE;
        } else {
            *out = g_string_sized_new(512);
        }

        if (ctx->prevsample == NULL) {
            ctx->prevsample = g_malloc0(logic->unitsize);
            if (ctx->prevsample == NULL) {
                g_string_free(*out, TRUE);
                *out = NULL;
                return SR_ERR_MALLOC;
            }
        }

        for (offset = 0; offset + logic->unitsize <= logic->length; offset += logic->unitsize) {
            sample = ((const uint8_t *)logic->data) + offset;
            timestamp_written = FALSE;

            for (bit_index = 0; bit_index < ctx->num_enabled_channels; bit_index++) {
                channel_index = ctx->channel_index[bit_index];
                current_bit = (sample[channel_index / 8] >> (channel_index % 8)) & 1;
                previous_bit = (ctx->prevsample[channel_index / 8] >> (channel_index % 8)) & 1;

                if (previous_bit == current_bit && ctx->samplecount > 0) {
                    continue;
                }

                if (!timestamp_written) {
                    g_string_append_printf(*out, "#%llu", (unsigned long long)ctx->samplecount);
                    timestamp_written = TRUE;
                }

                g_string_append_c(*out, ' ');
                g_string_append_c(*out, (char)('0' + current_bit));
                g_string_append_c(*out, (char)('!' + bit_index));
            }

            if (timestamp_written) {
                g_string_append_c(*out, '\n');
            }

            memcpy(ctx->prevsample, sample, logic->unitsize);
            ctx->samplecount++;
        }
        return SR_OK;
    case SR_DF_END:
        *out = g_string_sized_new(64);
        g_string_append_printf(*out, "#%llu\n", (unsigned long long)ctx->samplecount);
        return SR_OK;
    default:
        return SR_OK;
    }
}

static int minimal_vcd_cleanup(struct sr_output *o)
{
    struct minimal_vcd_context *ctx;

    if (o == NULL || o->priv == NULL) {
        return SR_ERR_ARG;
    }

    ctx = o->priv;
    g_free(ctx->prevsample);
    g_free(ctx->channel_index);
    g_free(ctx);
    o->priv = NULL;
    return SR_OK;
}

SR_PRIV struct sr_output_module output_vcd = {
    "vcd",
    "VCD",
    "Value Change Dump",
    output_vcd_exts,
    NULL,
    minimal_vcd_init,
    minimal_vcd_receive,
    minimal_vcd_cleanup,
};

SR_API const struct sr_output_module **sr_output_list(void)
{
    static const struct sr_output_module *modules[] = {
        &output_vcd,
        NULL,
    };

    return modules;
}

SR_API const struct sr_output_module *sr_output_find(char *id)
{
    if (id != NULL && strcmp(id, "vcd") == 0) {
        return &output_vcd;
    }

    return NULL;
}

SR_API const struct sr_output *sr_output_new(
    const struct sr_output_module *module,
    GHashTable *options,
    const struct sr_dev_inst *sdi
)
{
    struct sr_output *output;

    if (module == NULL || module->init == NULL) {
        return NULL;
    }

    output = g_malloc0(sizeof(*output));
    if (output == NULL) {
        return NULL;
    }

    output->module = module;
    output->sdi = sdi;
    if (module->init(output, options) != SR_OK) {
        g_free(output);
        return NULL;
    }

    return output;
}

SR_API int sr_output_send(const struct sr_output *output, const struct sr_datafeed_packet *packet, GString **out)
{
    if (output == NULL || output->module == NULL || output->module->receive == NULL) {
        return SR_ERR_ARG;
    }

    return output->module->receive(output, packet, out);
}

SR_API int sr_output_free(const struct sr_output *output)
{
    int status;

    if (output == NULL) {
        return SR_ERR_ARG;
    }

    status = SR_OK;
    if (output->module != NULL && output->module->cleanup != NULL) {
        status = output->module->cleanup((struct sr_output *)output);
    }

    g_free((gpointer)output);
    return status;
}
