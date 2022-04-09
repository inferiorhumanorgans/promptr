#include <sys/param.h>
#include <sys/queue.h>
#include <sys/socket.h>
#include <sys/sysctl.h>

#include <libprocstat.h>
#include <stdio.h>
#include <strings.h>
#include <unistd.h>

#include <limits>

#include "rust/cxx.h"

#define BUF_LEN 255

rust::String get_process_name(int64_t pid) {
    if (pid > std::numeric_limits<pid_t>::max()) {
        // Should probably print an error or something
        return std::string();
    }

    std::string ret_str;
    struct procstat *prstat = procstat_open_sysctl();
    unsigned int count = 0;
    struct kinfo_proc *kp = procstat_getprocs(prstat, KERN_PROC_PID, pid, &count);

    // Our parent process died?
    if (count == 0) {
        return std::string();
    }

    char pathbuf[BUF_LEN];
    bzero(pathbuf, BUF_LEN);

    int ret = procstat_getpathname(prstat, kp, pathbuf, BUF_LEN);
    
    if (ret == 0) {
        ret_str = std::string(pathbuf);
    } else {
        ret_str = std::string();
    }

    procstat_close(prstat);

    return ret_str;
}
