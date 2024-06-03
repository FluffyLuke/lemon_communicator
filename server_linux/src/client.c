#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>
#include <time.h>
#include "../includes/client.h"


void init_client(client_t* client, uv_tcp_t* stream) {
    static int64_t next_id = 0;

    struct sockaddr_in ipv4_addr;
    int namelen = sizeof(ipv4_addr);
    uv_tcp_getpeername(stream, (struct sockaddr*)&ipv4_addr, &namelen);

    client->id = next_id;
    client->addr = ipv4_addr;
    client->stream = (uv_stream_t*) stream;
    time(&client->last_time);

    next_id++;
}

void destroy_client(client_t* client) {
    uv_close((uv_handle_t*)client->stream, NULL);
}

void init_client_table(client_table_t* table, size_t init_size) {
    table->ptr = (client_t*)calloc(init_size, sizeof(client_t));
    table->len = init_size;
    table->used_len = 0;
}

client_t* get_client(client_table_t* table, size_t index) {
    if(index >= table->used_len) {
        return NULL;
    }
    return &table->ptr[index];
}

int8_t add_client(client_table_t* table, client_t client) {
    if (table->used_len == table->len) {
        table->len *= 2;
        table->ptr = realloc(table->ptr, table->len*CLIENT_SIZE);
    }
    table->ptr[table->used_len++] = client;
    return 0;
}

int8_t remove_client_index(client_table_t* table, size_t index) {
    if(index >= table->used_len) {
        return -1;
    }

    for (size_t j = index; j < (table)->used_len; j++) {
        (table)->ptr[j] = (table)->ptr[j+1];
    }
    (table)->used_len -= 1;
}

int8_t remove_client(client_table_t* table, client_t* client_to_remove) {
    return remove_client_index(table, (client_to_remove - table->ptr)/CLIENT_SIZE);

    // for(size_t i = 0; i < table->used_len; i++) {
    //     if(&table->ptr[i] == client_to_remove) {
    //         return remove_client_index(table, i);
    //     }
    // }
    // return -1;
}