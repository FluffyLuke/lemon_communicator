#ifndef __DATABASE
#define __DATABASE

#include "../includes/client.h"
#include "../../libs/api/includes/parser.hpp"
#include <stdbool.h>
#include <uv/unix.h>
#include <stdint.h>
#include "../../libs/mariadb-connector-c/include/mysql.h"

typedef enum {
    MARIADB,
} database_type;

#define MAX_BUFFER_LEN 4096
#define TOKEN_SIZE 32

typedef struct {
    char data[MAX_BUFFER_LEN];
    uint64_t length;
    int8_t is_null;
    int8_t is_hashed;
} token_t;

typedef struct {
    database_type db_type;
    char* host;
    char* user;
    char* password;
    uint16_t port;
    char* db_name;
    uint32_t options;
} db_driver_ctx;

typedef struct db_driver_t {
    uv_mutex_t lock;
    void* conn;
    db_driver_ctx database_ctx;
    client_t* (*get_all_clients)(struct db_driver_t*);
    token_t* (*login)(struct db_driver_t* db, client_t* client, char* key, char* password);
    bool (*check_token)(struct db_driver_t* db, client_t* client, token_t* token);
} db_driver_t;

int32_t init_database(db_driver_ctx db_ctx, db_driver_t* db);
void destroy_database(db_driver_t* db);

#endif