#include <uv.h>
#include "../includes/client.h"
#include "../../libs/api/includes/parser.hpp"
#include "../includes/server.h"
#include "../includes/database.h"
#include "../includes/requests.hpp"
#include "../../libs/openssl/include/crypto/sha.h"

void send(client_t* c, message_t* m) {
    char* ser_mes = serialize_message(m);

    uv_write_t* req = (uv_write_t*)malloc(sizeof(uv_write_t));
    uv_buf_t wrbuf = uv_buf_init(ser_mes, strlen(ser_mes));
    uv_write(req, c->stream, &wrbuf, 1, NULL);

    free(req);
    free(ser_mes);
}

void basic_res(server_ctx* ctx, client_t* client, message_status status, const char* err) {
    message_t res;
    init_message(&res, RESPONSE, status, err);
    send(client, &res);
    destroy_message(&res);
}

void ping_back(server_ctx* ctx, client_t* client, message_t* client_mes) {
    message_t res;
    init_message(&res, RESPONSE, client_mes->status, client_mes->err);
    send(client, &res);
    destroy_message(&res);
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
    db_driver_t* db = ctx->database;
    message_t res;
    printf("--1--\n");
    // db.login will init the message
    bool result = db->login(db, client_mes->data.login.key, client_mes->data.login.password, &res);
    printf("--2--\n");
    send(client, &res);
     printf("--3--\n");
    destroy_message(&res);
     printf("--4--\n");
}