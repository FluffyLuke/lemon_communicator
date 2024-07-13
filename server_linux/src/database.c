#include "../../libs/mariadb-connector-c/include/mysql.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <time.h>
#include <uv.h>
#include <uv/unix.h>
#include "../includes/client.h"
#include "../includes/database.h"
#include "../../libs/api/includes/parser.hpp"


client_t* mariadb_get_all_clients(db_driver_t* db) {
    // TODO end this later;
}

bool mariadb_login(db_driver_t* db, char* key, char* password, message_t* mes) {
    printf("--11--\n");
    uv_mutex_lock(&db->lock);
    printf("--12--\n");
    char* query = "\
    SELECT * FROM credentials \
    where key = ? and password = ?";
     printf("--13--\n");
    MYSQL_STMT* stmt = mysql_stmt_init((MYSQL*)db->conn);
     printf("--14--\n");
    mysql_stmt_prepare(stmt, query, -1);
     printf("--15--\n");
    MYSQL_BIND args[2];
    args[0].buffer_type = MYSQL_TYPE_STRING;
    args[0].buffer = key;
    args[0].buffer_length = sizeof(key);

    args[1].buffer_type = MYSQL_TYPE_STRING;
    args[1].buffer = password;
    args[1].buffer_length = sizeof(password);
     printf("--16--\n");
    mysql_stmt_bind_param(stmt, args);
     printf("--17--\n");
    mysql_stmt_execute(stmt);
    printf("--18--\n");
    bool if_ok;
    
    if(mysql_stmt_num_rows(stmt) == 0) {
        init_message(mes, LOGIN_RETURN, ERR, "Wrong credentials!");
        printf("--19--\n");
        mes->data.login_r.token = (char*)malloc(sizeof(1));
        printf("--110--\n");
        if_ok = false;
    } else {
        init_message(mes, LOGIN_RETURN, OK, NULL);
        printf("--111--\n");
        mes->data.login_r.token = (char*)malloc(sizeof(strlen("SUPER_KEY123")));
        printf("--112--\n");
        strcpy(mes->data.login_r.token, "SUPER_KEY123");
        printf("--113--\n");
        if_ok = true;
    }
    printf("--114--\n");
    mysql_stmt_close(stmt);
    printf("--115--\n");
    uv_mutex_unlock(&db->lock);
    printf("--116--\n");

    return if_ok;
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
            db->conn = conn;
            db->get_all_clients = mariadb_get_all_clients;
            db->login = mariadb_login;
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