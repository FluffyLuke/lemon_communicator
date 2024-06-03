#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

typedef struct {
    struct sockaddr_in addr;
    uv_stream_t* stream;
    time_t last_time;

    char name[32];
    char password[32];
} client_t;

#define CLIENT_SIZE (sizeof(client_t));

void init_client(client_t* client, uv_tcp_t* stream);
void destroy_client(client_t* client);

typedef struct {
    client_t* ptr;
    size_t len;
    size_t used_len;
} client_table_t;

void init_client_table(client_table_t* table, size_t init_size);
int8_t add_client(client_table_t* table, client_t client);
int8_t remove_client(client_table_t* table, size_t index);