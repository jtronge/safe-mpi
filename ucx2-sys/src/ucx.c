#include "ucx.h"

/* Wrapper around ucp_init() */
ucs_status_t rust_ucp_init(const ucp_params_t *params,
                           const ucp_config_t *config,
                           ucp_context_h *context_p)
{
    return ucp_init(params, config, context_p);
}

/* Various wrappers around macros */

int rust_ucs_ptr_is_ptr(const void *ptr)
{
    return UCS_PTR_IS_PTR(ptr);
}

ucs_status_t rust_ucs_ptr_status(const void *ptr)
{
    return UCS_PTR_STATUS(ptr);
}
