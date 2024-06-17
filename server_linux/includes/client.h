#ifndef __CLIENT
#define __CLIENT

#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>
#include <uv/unix.h>
#include "../../libs/vec/src/vec.h"

#define PASSWORD_LEN 50
#define NAME_LEN 50
#define EMAIL_LEN 255

typedef struct {
    int64_t id;
    uv_mutex_t lock;

    struct sockaddr_in addr;
    uv_stream_t* stream;

    char name[NAME_LEN];
    char password[PASSWORD_LEN];
    char email[EMAIL_LEN];
    char* session_token;
} client_t;

#define CLIENT_SIZE (sizeof(client_t))

void init_client(client_t* client, uv_tcp_t* stream);
void destroy_client(client_t* client);

typedef struct {
    client_t* ptr;
    size_t len;
    size_t used_len;
} client_table_t;

typedef vec_t(client_t*) client_vec_t;

typedef struct {
    uv_rwlock_t lock;
    client_vec_t vec;
} client_list_t;

void init_client_list(client_list_t* list);
void destroy_client_list(client_list_t* list);

#endif