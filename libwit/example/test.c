#include <stdio.h>

#include "wit.h"

int main(int argc, char *argv[]) {
    struct wit_context *context = wit_init(NULL);
    const char *result = wit_text_query(context, "hello", "WILND4RI5GQXXLHFZU6ASPH57HB3WIBR");
    printf("%s\n",result);
    return 0;
}
