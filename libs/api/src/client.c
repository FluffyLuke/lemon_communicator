#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>
#include <uv/unix.h>
#include "../includes/client.h"
#include "../includes/parser.hpp"

void init_client(client_t* client, uv_tcp_t* stream) {
    //static int64_t next_id = 0;
    struct sockaddr_in ipv4_addr;
    int namelen = sizeof(ipv4_addr);
    uv_tcp_getpeername(stream, (struct sockaddr*)&ipv4_addr, &namelen);

    //client->id = next_id;
    client->addr = ipv4_addr;
    client->stream = (uv_stream_t*) stream;
    client->logged = false;
    //next_id++;
    for(int32_t i = 0; i < FIRST_NAME_LEN; i++)
        client->first_name[i] = 0;
    for(int32_t i = 0; i < LAST_NAME_LEN; i++)
        client->last_name[i] = 0;
    for(int32_t i = 0; i < PASSWORD_LEN; i++)
        client->password[i] = 0;
    for(int32_t i = 0; i < EMAIL_LEN; i++)
        client->email[i] = 0;
    //client->session_token = NULL;

    uv_mutex_init(&client->lock);
}


void destroy_client(client_t* client) {
    uv_mutex_destroy(&client->lock);
    uv_close((uv_handle_t*)client->stream, NULL);
}

void init_client_list(client_list_t* list) {
    list->lock = (uv_rwlock_t*)malloc(sizeof(uv_rwlock_t));
    list->vec = (client_vec_t*)malloc(sizeof(client_vec_t));
    vec_init(list->vec);
    uv_rwlock_init(list->lock);
}

void destroy_client_list(client_list_t* list) {
    // client_t val; size_t i;
    // vec_foreach(&list->vec, val, i) {
    //     destroy_client(&val);
    // }
    vec_deinit(list->vec);
    uv_rwlock_destroy(list->lock);
    free(list->lock);
    free(list->vec);
}

void client_to_client_body(client_t* client, client_body_t* client_body) {
    memcpy(client_body->email, client->email, EMAIL_LEN);
    memcpy(client_body->first_name, client->first_name, FIRST_NAME_LEN);
    memcpy(client_body->last_name, client->last_name, LAST_NAME_LEN);
    client_body->ip = client->addr;
}