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

uint32_t count_node_num(const char *path_cstr);

char *generate_challenge(void);

char *generate_replica_id(void);

char *generate_sample_file(uint32_t expected_size, const char *path_cstr);

char *generate_setup_params(uint32_t node_num);

char *generate_store_config(uint32_t node_num, const char *dir_cstr);

char *initialize_target_dir(const char *dir_cstr, bool need_clean);

char *porep_prove(const char *replica_path_cstr,
                  const char *sp_data_cstr,
                  const char *replica_id_cstr,
                  const char *chal_cstr,
                  const char *proof_path_cstr);

char *porep_setup(const char *src_path_cstr,
                  const char *sp_data_cstr,
                  const char *scfg_data_cstr,
                  const char *replica_id_cstr);

char *porep_verify(const char *replica_path_cstr,
                   const char *sp_data_cstr,
                   const char *replica_id_cstr,
                   const char *chal_cstr,
                   const char *proof_path_cstr);

void release(char *s);

#endif /* wrapper_H */

#ifdef __cplusplus
} /* extern "C" */
#endif
