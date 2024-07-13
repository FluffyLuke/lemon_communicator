#ifndef __REQUESTS
#define __REQUESTS

#include <uv.h>
#include "../includes/server.h"

void ping_back(server_ctx* ctx, client_t* client, message_t* client_mes);
void basic_res(server_ctx* ctx, client_t* client, message_status status, const char* err);
void login_user(server_ctx* ctx, client_t* client, message_t* client_mes);

#endif