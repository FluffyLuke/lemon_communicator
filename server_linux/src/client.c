#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>
#include <time.h>
#include "../includes/client.h"

void init_client(client_t* client, uv_tcp_t* stream) {
    struct sockaddr_in ipv4_addr;
    int namelen = sizeof(ipv4_addr);
    uv_tcp_getpeername(stream, (struct sockaddr*)&ipv4_addr, &namelen);

    client->addr = ipv4_addr;
    client->stream = go(uv_stream_t*) stream;
    time(&client->last_time);
}

void destroy_client(client_t* client) {
    uv_close((uv_handle_t*)client->stream, NULL);
    free(client);
}

void init_client_table(client_table_t* table, size_t init_size) {
    table->ptr = (client_t*)calloc(init_size, sizeof(client_t*));
    table->len = init_size;
    table->used_len = 0;
}

int8_t add_client(client_table_t* table, client_t client) {
    if(table->len <= table->used_len) {
        // adds place for 10 more clients
        table->ptr = realloc(table->ptr, (sizeof(client_t)*table->len)+10);
        if(table->ptr == NULL) {
            return -1;
        }
    }
    // find next empty spot in array
    client_t* next_place = table->ptr + (sizeof(client_t)*table->used_len);
    mempcpy(next_place, &client, sizeof(client));
    return 0;
}

int8_t remove_client(client_table_t* table, size_t index) {
    if(index >= table->len) {
        return -1;
    }
    if(index < 0) {
        return -1;
    }
    client_t* client_to_remove = table->ptr + (sizeof(client_t)*index);
    size_t cs = CLIENT_SIZE;
    size_t clients_after_size = ((table->used_len)-(index+1))*CLIENT_SIZE;
    memcpy(client_to_remove, client_to_remove+cs, clients_after_size);
}