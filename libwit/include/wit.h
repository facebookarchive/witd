#ifndef WIT_H
#define WIT_H

#include <stdlib.h>

struct wit_handle;

struct wit_handle *wit_init(const char *device_opt);
struct wit_handle *wit_start_recording(struct wit_handle *handle, const char *access_token);

#endif
