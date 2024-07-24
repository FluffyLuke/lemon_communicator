#ifndef __DATABASE
#define __DATABASE

#include "../../libs/api/includes/client.h"
#include "../../libs/api/includes/parser.hpp"
#include <stdbool.h>
#include <uv/unix.h>
#include <stdint.h>
#include "../../libs/mariadb-connector-c/include/mysql.h"

typedef enum {
    MARIADB,
} database_type;

typedef struct {
    char data[TOKEN_BUFFER_SIZE];
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

// TODO change all booleans to simple ints
typedef struct db_driver_t {
    uv_mutex_t lock;
    void* conn;
    db_driver_ctx database_ctx;
    client_t* (*get_all_clients)(struct db_driver_t*);
    token_t* (*login)(struct db_driver_t* db, client_t* client, char* key, char* password);
    bool (*check_token)(struct db_driver_t* db, client_t* client, token_t* token);
    int8_t (*init_client_data)(struct db_driver_t* db, client_t* client);
} db_driver_t;

void init_token(token_t* token, char* token_raw, size_t len);
int32_t init_database(db_driver_ctx db_ctx, db_driver_t* db);
void destroy_database(db_driver_t* db);

#endif