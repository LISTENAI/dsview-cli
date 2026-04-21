#ifndef DSVIEW_SYS_COMPAT_MSVC_PREINCLUDE_H
#define DSVIEW_SYS_COMPAT_MSVC_PREINCLUDE_H

#if defined(_WIN32)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN 1
#endif

#ifndef NOMINMAX
#define NOMINMAX 1
#endif

#include <winsock2.h>
#include <ws2tcpip.h>

/*
 * DSView's log.h uses GCC-style variadic macros which MSVC does not parse.
 * Predeclare the same API with standard __VA_ARGS__ macros and suppress the
 * original header body via its include guard.
 */
#ifndef _SR_LOG_H_
#define _SR_LOG_H_

#include "libsigrok-internal.h"

#ifdef LIBUSB_CALL
#undef LIBUSB_CALL
#define LIBUSB_CALL
#endif

#ifdef HAVE_LA_DEMO
#undef HAVE_LA_DEMO
#endif

#ifndef strcasecmp
#define strcasecmp _stricmp
#endif

#include <log/xlog.h>

extern xlog_writer *sr_log;

SR_PRIV void sr_log_init(void);
SR_PRIV void sr_log_uninit(void);
SR_API void ds_log_set_context(xlog_context *ctx);
SR_API void ds_log_level(int level);

#define LOG_PREFIX ""
#define sr_err(...) xlog_err(sr_log, LOG_PREFIX __VA_ARGS__)
#define sr_warn(...) xlog_warn(sr_log, LOG_PREFIX __VA_ARGS__)
#define sr_info(...) xlog_info(sr_log, LOG_PREFIX __VA_ARGS__)
#define sr_dbg(...) xlog_dbg(sr_log, LOG_PREFIX __VA_ARGS__)
#define sr_detail(...) xlog_detail(sr_log, LOG_PREFIX __VA_ARGS__)

#endif
#endif

#endif
