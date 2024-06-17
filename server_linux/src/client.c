#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>
#include <uv/unix.h>
#include "../includes/server.h"


// TODO make user decide, whether they want client on heap or stack
void init_client(client_t* client, uv_tcp_t* stream) {
    static int64_t next_id = 0;

    struct sockaddr_in ipv4_addr;
    int namelen = sizeof(ipv4_addr);
    uv_tcp_getpeername(stream, (struct sockaddr*)&ipv4_addr, &namelen);

    client->id = next_id;
    client->addr = ipv4_addr;
    client->stream = (uv_stream_t*) stream;
    next_id++;

    for(int32_t i = 0; i < NAME_LEN; i++)
        client->name[i] = 0;
    for(int32_t i = 0; i < PASSWORD_LEN; i++)
        client->password[i] = 0;
    for(int32_t i = 0; i < EMAIL_LEN; i++)
        client->email[i] = 0;
    client->session_token = NULL;

    uv_mutex_init(&client->lock);
}


void destroy_client(client_t* client) {
    uv_mutex_init(&client->lock);
    uv_close((uv_handle_t*)client->stream, NULL);
}

void init_client_list(client_list_t* list) {
    vec_init(list);
    uv_rwlock_init(&list->lock);
}

void destroy_client_list(client_list_t* list) {
    // client_t val; size_t i;
    // vec_foreach(&list->vec, val, i) {
    //     destroy_client(&val);
    // }
    vec_deinit(&list->vec);
    uv_rwlock_destroy(&list->lock);
}