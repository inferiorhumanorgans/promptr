#include <libproc.h>
#include <strings.h>
#include <unistd.h>

#include <limits>

#include "rust/cxx.h"

#define BUF_LEN 256

rust::String get_process_name(int64_t pid) {
    // https://stackoverflow.com/questions/12841488/how-can-i-determine-the-maximum-value-of-a-pid-t
    if (pid > std::numeric_limits<pid_t>::max()) {
        // Should probably print an error or something
        return std::string();
    }

    char buf[BUF_LEN];
    bzero(buf, BUF_LEN);

    int len = proc_name(pid, &buf, BUF_LEN);

    if (len) {
        return std::string(buf);
    } else {
        return std::string();
    }
}
