// runner.c: to validate the rust library 
#include <stdio.h>
#include <stdint.h>

extern void sentinel(const char *name);
extern uint32_t setup(const char *data_path, const char *cache_dir);

int main(void) {
    sentinel("tjan@runner.c");

    uint32_t res = setup("rust/sample/sample.txt", "rust/sample");
    printf("setup() returns %d\n", res);
}