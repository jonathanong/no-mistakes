#include <stddef.h>
#include <stdlib.h>

int main(void) {
  // More than 65,536 events exercise real concatenated memtrack zstd frames.
  for (size_t index = 0; index < 40000; index++) {
    volatile unsigned char *value = malloc(16);
    if (!value) abort();
    value[0] = (unsigned char)index;
    free((void *)value);
  }
  return 0;
}
