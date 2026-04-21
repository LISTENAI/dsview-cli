#ifndef DSVIEW_SYS_COMPAT_SYS_TIME_H
#define DSVIEW_SYS_COMPAT_SYS_TIME_H

#if defined(_WIN32)
#include <stdint.h>
#include <winsock2.h>
#include <windows.h>

static __inline int gettimeofday(struct timeval *tv, void *tz) {
    FILETIME file_time;
    ULARGE_INTEGER time_value;

    (void)tz;
    if (tv == NULL) {
        return -1;
    }

    GetSystemTimeAsFileTime(&file_time);
    time_value.LowPart = file_time.dwLowDateTime;
    time_value.HighPart = file_time.dwHighDateTime;
    time_value.QuadPart -= 116444736000000000ULL;

    tv->tv_sec = (long)(time_value.QuadPart / 10000000ULL);
    tv->tv_usec = (long)((time_value.QuadPart % 10000000ULL) / 10ULL);
    return 0;
}
#else
#include_next <sys/time.h>
#endif

#endif
