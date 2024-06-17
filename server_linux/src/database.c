#include "../../libs/mariadb-connector-c/include/mysql.h"
#include <stdbool.h>
#include <stdint.h>
#include <time.h>
#include <uv.h>
#include <uv/unix.h>
#include "../includes/client.h"
#include "../includes/database.h"



client_t* mariadb_get_all_clients(db_driver_t* db) {
    
}

// TODO add a pool of connections, instead of using a single one
int32_t init_database(db_driver_ctx db_ctx, db_driver_t* db) {
    int32_t result;
    db->database_ctx = db_ctx;
    uv_mutex_init(&db->lock);
    switch(db_ctx.db_type) {
        case MARIADB:
            MYSQL* conn;
            if(!(conn = mysql_init(0))){
                fprintf(stderr, "Cannot create MYSQL struct\n");
                return -1;
            }
            if(!db_ctx.host || !db_ctx.user || !db_ctx.password || !db_ctx.db_name || db_ctx.port == 0) {
                fprintf(stderr, "Not enough options for database creation\n");
                return -1;
            }
            // printf("%d\n", db_ctx.db_type);
            // printf("%s\n", db_ctx.host);
            // printf("%s\n", db_ctx.user);
            // printf("%s\n", db_ctx.password);
            // printf("%s\n", db_ctx.db_name);
            // printf("%d\n", db_ctx.port);

            if(!mysql_real_connect(
                conn, 
                db_ctx.host, 
                db_ctx.user, 
                db_ctx.password, 
                db_ctx.db_name,
                db_ctx.port,
                NULL,
                0
            )) {
                fprintf(stderr, "Cannot connect to database!\n");
                db->conn = NULL;
                return -1;
            }

            db->get_all_clients = mariadb_get_all_clients;
            
            return 0;
            break;
    }
}

void destroy_database(db_driver_t* db) {
    switch (db->database_ctx.db_type) {
        case MARIADB:
            mysql_close((MYSQL*)db->conn);
            break;
    }
    uv_mutex_destroy(&db->lock);
}