#ifndef __REQUESTS
#define __REQUESTS

#include <uv.h>
#include "../includes/server.h"

void ping_back(uv_stream_t* stream, message_t* client_mes);
void basic_res(uv_stream_t* stream, message_status status, const char* err);
void register_client(uv_stream_t* stream, message_t* client_mes);
void login_user(server_ctx* ctx, client_t* client, message_t* client_mes);

#endif