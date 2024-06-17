#ifndef __DATABASE
#define __DATABASE

#include "../includes/client.h"
#include <uv/unix.h>

typedef enum {
    MARIADB,
} database_type;

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
} db_driver_t;

int32_t init_database(db_driver_ctx db_ctx, db_driver_t* db);
void destroy_database(db_driver_t* db);

#endif