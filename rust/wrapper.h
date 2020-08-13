/* wrapper Header */
#ifdef __cplusplus
extern "C" {
#endif


#ifndef wrapper_H
#define wrapper_H

/* Generated with cbindgen:0.14.3 */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

void sentinel(const char *name);

uint32_t setup(const char *data_path, const char *cache_dir);

#endif /* wrapper_H */

#ifdef __cplusplus
} /* extern "C" */
#endif
