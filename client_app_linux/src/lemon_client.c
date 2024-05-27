#include "./includes/lemon_ctx.h"
#include "./includes/utils.h"
#include <string.h>
#include <uv.h>

bool try_set_credentials(lemon_client_ctx* ctx, const char* new_name, const char* new_password) {
    if (strlen(new_name) > NAME_LENGHT) return false;
    if (strlen(new_password) > PASSWORD_LENGHT) return false;

    if(uv_rwlock_tryrdlock(&ctx->lock) != 0)
        return false;

    strcpy(ctx->name, new_name);
    strcpy(ctx->password, new_password);
    
    uv_rwlock_rdunlock(&ctx->lock);

    return true;
}