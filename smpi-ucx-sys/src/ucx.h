#include <ucp/api/ucp.h>
#include <ucm/api/ucm.h>

ucs_status_t rust_ucp_init(const ucp_params_t *params,
                           const ucp_config_t *config,
                           ucp_context_h *context_p);
/* Functional macro wrappers */
int rust_ucs_ptr_is_ptr(const void *ptr);
int rust_ucs_ptr_is_err(const void *ptr);
ucs_status_t rust_ucs_ptr_status(const void *ptr);
size_t rust_ucp_dt_make_contig(size_t sz);
