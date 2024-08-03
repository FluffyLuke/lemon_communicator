#ifndef __SERVER_TCP
#define __SERVER_TCP

#include "lemon_ctx.h"
int32_t connect_to_server(lemon_ctx* ctx, struct sockaddr_in* addr);
void tcp_loop(uv_work_t* req);

#endif