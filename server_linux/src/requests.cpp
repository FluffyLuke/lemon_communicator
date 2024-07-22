#include <cstddef>
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

void login_user(server_ctx* ctx, client_t* client, message_t* client_mes) {
    db_driver_t* db = ctx->database;
    message_t res;

    // db.login will init the message
    token_t* token = db->login(db, client, client_mes->data.login.key, client_mes->data.login.password);
    char* sliced_token = (char*)malloc(TOKEN_BUFFER_SIZE);
    memset(sliced_token, 0, TOKEN_BUFFER_SIZE);
    strncpy(sliced_token, token->data, token->length);

    if(token != NULL) {
        init_message(&res, LOGIN_RETURN, OK, NULL);
        res.data.login_r.token = sliced_token;
        client->logged = true;
    } else {
        init_message(&res, LOGIN_RETURN, ERR, "Cannot login client");
        res.data.login_r.token = NULL;
    }

    if(!db->init_client_data(db, client))
        printf("Cannot access client data!");
    
    send(client, &res);
    free(token);
    destroy_message(&res);
}


void get_network_state(server_ctx* ctx, client_t* client, message_t* client_mes) {
    client_list_t* cl = ctx->client_list;
    db_driver_t* db = ctx->database;
    message_t res;
    
    client_body_vec_t logged_client_bodies;
    vec_init(&logged_client_bodies);
    res.data.network_r.clients = logged_client_bodies;

    token_t token;
    bool result;
    // TODO find solution for replacing "strlen"
    init_token(&token, client_mes->data.network.token, strlen(client_mes->data.network.token));
    token.is_hashed = false;
    result = db->check_token(db, client, &token);

    printf("CHECK TOKEN RESULT: %d\n", result);
    if(!result) {
        init_message(&res, NETWORK_STATE_RETURN, ERR, "Token is invalid");
        send(client, &res);
        destroy_message(&res);
        printf("CHECK TOKEN RESULT: %d\n", result);
        return;
    }

    uv_rwlock_rdlock(cl->lock);
    client_t* c;
    client_body_t cb;
    int32_t i;
    vec_foreach(cl->vec, c, i) {
        printf("CLIENT\n");
        if(c->logged == true) {
            printf("LOGGED CLIENT\n");
            client_to_client_body(c, &cb);
            vec_push(&logged_client_bodies, cb);
        }
    }
    uv_rwlock_rdunlock(cl->lock);

    // Second assigment is needed, since changes accured in the vec
    res.data.network_r.clients = logged_client_bodies;
    init_message(&res, NETWORK_STATE_RETURN, OK, NULL);

    send(client, &res);
    destroy_message(&res);
}