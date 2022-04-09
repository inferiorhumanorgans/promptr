#pragma once
#include "promptr/src/lib.rs.h"
#include "rust/cxx.h"

rust::String get_process_name(int64_t pid);
