#ifndef __LEMON_SERVER
#define __LEMON_SERVER

#include <uv.h>

#include <uv/unix.h>
#include "../../libs/api/includes/parser.hpp"
#include "../../libs/sqlite/sqlite3.h"
#include "../includes/database.h"
#include "../../libs/vec/src/vec.h"

typedef struct {
    uint16_t port;
    uv_loop_t* loop;
    
    client_list_t client_list;
    db_driver_t* database;
} server_ctx;


#define SERVER_PORT_ARG "--server-port"
#define DATABASE_HOST_ARG "--db-host"
#define DATABASE_USER_ARG "--db-user"
#define DATABASE_PASSWORD_ARG "--db-password"
#define DATABASE_PORT_ARG "--db-port"
#define DATABASE_NAME_ARG "--db-name"

#define DEFAULT_DATABASE_PASSWORD ""
#define DEFAULT_DATABASE_HOST "localhost"
#define DEFAULT_DATABASE_NAME "lemon_comm"
#define DEFAULT_SERVER_PORT 22005



#endif