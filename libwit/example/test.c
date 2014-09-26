#include <stdio.h>
#include <unistd.h>

#include "wit.h"

int main(int argc, char *argv[]) {
    struct wit_context *context = wit_init(NULL);
    wit_text_query(context, "hello", "MPZX6ST6I7R4LK7SWBYAUK5MDTGC5I4A");
    const char *result = wit_text_query(context, "hello world", "MPZX6ST6I7R4LK7SWBYAUK5MDTGC5I4A");
    /*wit_start_recording(context, "MPZX6ST6I7R4LK7SWBYAUK5MDTGC5I4A");
    sleep(2);
    const char *result = wit_stop_recording(context);*/
    printf("%s\n",result);
    return 0;
}
