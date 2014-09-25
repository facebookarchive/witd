#ifndef WIT_H
#define WIT_H

#include <stdlib.h>

struct wit_context;

struct wit_context *wit_init(const char *device_opt);
void wit_start_recording(struct wit_context *context, const char *access_token);
const char *wit_stop_recording(struct wit_context *context);
const char *wit_text_query(struct wit_context *context, const char *text, const char *access_token);

#endif
