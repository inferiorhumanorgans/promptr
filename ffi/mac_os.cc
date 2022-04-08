#include <libproc.h>
#include <strings.h>
#include <unistd.h>

#include "rust/cxx.h"

#define BUF_LEN 256

rust::String get_process_name(uint64_t pid) {
    char buf[BUF_LEN];
    bzero(buf, BUF_LEN);

    int len = proc_name(pid, &buf, BUF_LEN);

    if (len) {
        return std::string(buf);
    } else {
        return std::string();
    }
}
