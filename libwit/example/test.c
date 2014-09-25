#include "wit.h"

int main(int argc, char *argv[]) {
    struct wit_handle *handle = wit_init(NULL);
    wit_start_recording(handle, "WILND4RI5GQXXLHFZU6ASPH57HB3WIBR");
    while (1) {}
    return 0;
}
