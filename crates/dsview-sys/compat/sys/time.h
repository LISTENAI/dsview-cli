#ifndef DSVIEW_SYS_COMPAT_SYS_TIME_H
#define DSVIEW_SYS_COMPAT_SYS_TIME_H

#if defined(_WIN32)
#include <winsock2.h>
#else
#include_next <sys/time.h>
#endif

#endif
