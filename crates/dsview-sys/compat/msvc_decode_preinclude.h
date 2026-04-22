#ifndef DSVIEW_SYS_COMPAT_MSVC_DECODE_PREINCLUDE_H
#define DSVIEW_SYS_COMPAT_MSVC_DECODE_PREINCLUDE_H

#if defined(_WIN32)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN 1
#endif

#ifndef NOMINMAX
#define NOMINMAX 1
#endif

#include <BaseTsd.h>
#include <winsock2.h>
#include <ws2tcpip.h>

#ifndef _SSIZE_T_DEFINED
typedef SSIZE_T ssize_t;
#define _SSIZE_T_DEFINED
#endif

#ifndef strcasecmp
#define strcasecmp _stricmp
#endif

/*
 * libsigrokdecode4DSL/log.h uses GCC-style variadic macros which MSVC does not
 * parse. Predeclare equivalent __VA_ARGS__ macros and suppress the original
 * header body via its include guard.
 */
#ifndef _SRD_LOG_H_
#define _SRD_LOG_H_

#include <log/xlog.h>
#include "libsigrokdecode.h"

extern xlog_writer *srd_log;

SRD_PRIV void srd_log_init(void);
SRD_PRIV void srd_log_uninit(void);
SRD_API void srd_log_set_context(xlog_context *ctx);
SRD_API void srd_log_level(int level);

#define LOG_PREFIX ""
#define srd_err(...) xlog_err(srd_log, LOG_PREFIX __VA_ARGS__)
#define srd_warn(...) xlog_warn(srd_log, LOG_PREFIX __VA_ARGS__)
#define srd_info(...) xlog_info(srd_log, LOG_PREFIX __VA_ARGS__)
#define srd_dbg(...) xlog_dbg(srd_log, LOG_PREFIX __VA_ARGS__)
#define srd_detail(...) xlog_detail(srd_log, LOG_PREFIX __VA_ARGS__)

#endif
#endif

#endif
