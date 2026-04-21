#ifndef DSVIEW_SYS_COMPAT_UNISTD_H
#define DSVIEW_SYS_COMPAT_UNISTD_H

#if defined(_WIN32)
#include <BaseTsd.h>
#include <io.h>
#include <process.h>
#include <windows.h>

typedef SSIZE_T ssize_t;
typedef int pid_t;

static __inline int usleep(unsigned int microseconds) {
    Sleep((microseconds + 999U) / 1000U);
    return 0;
}

#ifndef close
#define close _close
#endif

#ifndef read
#define read _read
#endif

#ifndef write
#define write _write
#endif

#ifndef lseek
#define lseek _lseek
#endif

#ifndef unlink
#define unlink _unlink
#endif

#ifndef access
#define access _access
#endif

#ifndef getpid
#define getpid _getpid
#endif

#ifndef fileno
#define fileno _fileno
#endif
#else
#include_next <unistd.h>
#endif

#endif
