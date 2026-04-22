#include "libsigrok-internal.h"
#include <glib.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "config.h"

#undef LOG_PREFIX
#define LOG_PREFIX "vcd: "

struct vcd_context {
    int num_enabled_channels;
    uint8_t *prevsample;
    gboolean header_done;
    int period;
    int *channel_index;
    uint64_t samplerate;
    uint64_t samplecount;
};

static int vcd_init(struct sr_output *output, GHashTable *options)
{
    struct vcd_context *ctx;
    struct sr_channel *channel;
    GSList *node;
    int num_enabled_channels;
    int index;

    (void)options;

    num_enabled_channels = 0;
    for (node = output->sdi->channels; node != NULL; node = node->next) {
        channel = node->data;
        if (channel->type == SR_CHANNEL_LOGIC && channel->enabled) {
            num_enabled_channels++;
        }
    }
    if (num_enabled_channels > 94) {
        sr_err("VCD only supports 94 channels.");
        return SR_ERR;
    }

    ctx = malloc(sizeof(*ctx));
    if (ctx == NULL) {
        sr_err("%s,ERROR:failed to alloc memory.", __func__);
        return SR_ERR_MALLOC;
    }
    memset(ctx, 0, sizeof(*ctx));

    output->priv = ctx;
    ctx->num_enabled_channels = num_enabled_channels;
    ctx->channel_index = malloc(sizeof(int) * ctx->num_enabled_channels);
    if (ctx->channel_index == NULL) {
        free(ctx);
        output->priv = NULL;
        sr_err("%s,ERROR:failed to alloc memory.", __func__);
        return SR_ERR_MALLOC;
    }

    for (index = 0, node = output->sdi->channels; node != NULL; node = node->next) {
        channel = node->data;
        if (channel->type != SR_CHANNEL_LOGIC || !channel->enabled) {
            continue;
        }
        ctx->channel_index[index++] = channel->index;
    }

    return SR_OK;
}

static GString *vcd_header(const struct sr_output *output)
{
    struct vcd_context *ctx = output->priv;
    struct sr_channel *channel;
    GVariant *variant;
    GString *header;
    GSList *node;
    time_t now;
    int num_channels;
    int signal_index;
    char *samplerate_string;
    char *period_string;
    char *timestamp;

    header = g_string_sized_new(512);
    num_channels = g_slist_length(output->sdi->channels);

    now = time(NULL);
    timestamp = g_strdup(ctime(&now));
    timestamp[strlen(timestamp) - 1] = 0;
    g_string_printf(header, "$date %s $end\n", timestamp);
    g_free(timestamp);

    g_string_append_printf(header, "$version %s %s $end\n", PACKAGE, PACKAGE_VERSION);
    g_string_append_printf(
        header,
        "$comment\n  Acquisition with %d/%d channels",
        ctx->num_enabled_channels,
        num_channels
    );

    if (ctx->samplerate == 0) {
        if (sr_config_get(output->sdi->driver, output->sdi, NULL, NULL, SR_CONF_SAMPLERATE, &variant) == SR_OK) {
            if (variant != NULL) {
                ctx->samplerate = g_variant_get_uint64(variant);
                g_variant_unref(variant);
            }
        }
    }
    if (ctx->samplerate != 0) {
        samplerate_string = sr_samplerate_string(ctx->samplerate);
        g_string_append_printf(header, " at %s", samplerate_string);
        g_free(samplerate_string);
    }
    g_string_append_printf(header, "\n$end\n");

    if (ctx->samplerate > SR_MHZ(1)) {
        ctx->period = SR_GHZ(1);
    } else if (ctx->samplerate > SR_KHZ(1)) {
        ctx->period = SR_MHZ(1);
    } else {
        ctx->period = SR_KHZ(1);
    }
    period_string = sr_period_string(ctx->period);
    g_string_append_printf(header, "$timescale %s $end\n", period_string);
    g_free(period_string);

    g_string_append_printf(header, "$scope module %s $end\n", PACKAGE);

    signal_index = 0;
    for (node = output->sdi->channels; node != NULL; node = node->next) {
        channel = node->data;
        if (channel->type != SR_CHANNEL_LOGIC || !channel->enabled) {
            continue;
        }
        g_string_append_printf(header, "$var wire 1 %c %s $end\n", (char)('!' + signal_index), channel->name);
        signal_index++;
    }

    g_string_append(header, "$upscope $end\n$enddefinitions $end\n");
    return header;
}

static int vcd_receive(const struct sr_output *output, const struct sr_datafeed_packet *packet, GString **out)
{
    const struct sr_datafeed_meta *meta;
    const struct sr_datafeed_logic *logic;
    const struct sr_config *config;
    GSList *node;
    struct vcd_context *ctx;
    unsigned int offset;
    int signal_index;
    int bit_index;
    int current_bit;
    int previous_bit;
    uint8_t *sample;
    gboolean timestamp_written;

    *out = NULL;
    if (output == NULL || output->priv == NULL) {
        return SR_ERR_BUG;
    }

    ctx = output->priv;
    switch (packet->type) {
    case SR_DF_META:
        meta = packet->payload;
        for (node = meta->config; node != NULL; node = node->next) {
            config = node->data;
            if (config->key == SR_CONF_SAMPLERATE) {
                ctx->samplerate = g_variant_get_uint64(config->data);
            }
        }
        break;
    case SR_DF_LOGIC:
        logic = packet->payload;
        if (!ctx->header_done) {
            *out = vcd_header(output);
            ctx->header_done = TRUE;
        } else {
            *out = g_string_sized_new(512);
        }

        if (ctx->prevsample == NULL) {
            ctx->prevsample = malloc(logic->unitsize);
            if (ctx->prevsample == NULL) {
                sr_err("%s,ERROR:failed to alloc memory.", __func__);
                g_string_free(*out, TRUE);
                *out = NULL;
                return SR_ERR_MALLOC;
            }
            memset(ctx->prevsample, 0, logic->unitsize);
        }

        for (offset = 0; offset <= logic->length - logic->unitsize; offset += logic->unitsize) {
            sample = ((uint8_t *)logic->data) + offset;
            timestamp_written = FALSE;

            for (signal_index = 0; signal_index < ctx->num_enabled_channels; signal_index++) {
                bit_index = signal_index;
                current_bit = ((unsigned)sample[bit_index / 8] >> (bit_index % 8)) & 1;
                previous_bit = ((unsigned)ctx->prevsample[bit_index / 8] >> (bit_index % 8)) & 1;

                if (previous_bit == current_bit && ctx->samplecount > 0) {
                    continue;
                }

                if (!timestamp_written) {
                    g_string_append_printf(
                        *out,
                        "#%.0f",
                        (double)ctx->samplecount / ctx->samplerate * ctx->period
                    );
                }

                g_string_append_c(*out, ' ');
                g_string_append_c(*out, '0' + current_bit);
                g_string_append_c(*out, '!' + signal_index);
                timestamp_written = TRUE;
            }

            if (timestamp_written) {
                g_string_append_c(*out, '\n');
            }

            ctx->samplecount++;
            memcpy(ctx->prevsample, sample, logic->unitsize);
        }
        break;
    case SR_DF_END:
        *out = g_string_sized_new(512);
        g_string_printf(
            *out,
            "#%.0f\n",
            (double)ctx->samplecount / ctx->samplerate * ctx->period
        );
        break;
    default:
        break;
    }

    return SR_OK;
}

static int vcd_cleanup(struct sr_output *output)
{
    struct vcd_context *ctx;

    if (output == NULL || output->priv == NULL) {
        return SR_ERR_ARG;
    }

    ctx = output->priv;
    g_free(ctx->prevsample);
    g_free(ctx->channel_index);
    g_free(ctx);
    return SR_OK;
}

struct sr_output_module output_vcd = {
    .id = "vcd",
    .name = "VCD",
    .desc = "Value Change Dump",
    .exts = (const char *[]){"vcd", NULL},
    .options = NULL,
    .init = vcd_init,
    .receive = vcd_receive,
    .cleanup = vcd_cleanup,
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

    if (module == NULL) {
        return NULL;
    }

    output = calloc(1, sizeof(*output));
    if (output == NULL) {
        return NULL;
    }

    output->module = module;
    output->sdi = sdi;
    if (module->init != NULL && module->init(output, options) != SR_OK) {
        free(output);
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
    int status = SR_OK;

    if (output == NULL) {
        return SR_ERR_ARG;
    }

    if (output->module != NULL && output->module->cleanup != NULL) {
        status = output->module->cleanup((struct sr_output *)output);
    }

    free((void *)output);
    return status;
}
