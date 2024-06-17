#include <uv.h>
#include "../includes/client.h"
#include "../../libs/api/includes/parser.hpp"
#include "../includes/server.h"

void basic_res(server_ctx* ctx, client_t* client, message_status status, const char* err) {
    message_t* res = (message_t*)malloc(sizeof(message_t));
    init_message(res, status, err);
    char* ser_mes = serialize_message(res);

    uv_write_t* req = (uv_write_t*)malloc(sizeof(uv_write_t));
    uv_buf_t wrbuf = uv_buf_init(ser_mes, strlen(ser_mes));
    uv_write(req, client->stream, &wrbuf, 1, NULL);

    free(req);
    free(ser_mes);
    destroy_message(res);
}

void ping_back(server_ctx* ctx, client_t* client, message_t* client_mes) {
    message_t* res = (message_t*)malloc(sizeof(message_t));
    init_message(res, client_mes->status, client_mes->err);
    char* ser_mes = serialize_message(res);

    uv_write_t* req = (uv_write_t*)malloc(sizeof(uv_write_t));
    uv_buf_t wrbuf = uv_buf_init(ser_mes, strlen(ser_mes));
    uv_write(req, client->stream, &wrbuf, 1, NULL);

    free(req);
    free(ser_mes);
    destroy_message(res);
}

// void register_client(uv_stream_t* stream, message* client_mes) {
//     // TODO create real registration for client...

//     message* res = create_response(client_mes->status, client_mes->err);
//     char* ser_mes = serialize_response(res);

//     uv_write_t* req = (uv_write_t*)malloc(sizeof(uv_write_t));
//     uv_buf_t wrbuf = uv_buf_init(ser_mes, strlen(ser_mes));
//     uv_write(req, stream, &wrbuf, 1, NULL);

//     free(req);
//     destroy_message(res);
// }

void login_user(server_ctx* ctx, client_t* client, message_t* client_mes) {
    uv_mutex_lock(&ctx->database->lock);

    

    uv_mutex_unlock(&ctx->database->lock);
}