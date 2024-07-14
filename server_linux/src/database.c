#include "../../libs/mariadb-connector-c/include/mysql.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/time.h>
#include <uv.h>
#include <uv/unix.h>
#include "../includes/client.h"
#include "../includes/database.h"
#include "../../libs/api/includes/parser.hpp"
#include <openssl/sha.h>

#define TOKEN_SIZE 32

char* rand_string(size_t length) {
    static char charset[] = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789,.-#'?!";        
    char* rand_string = NULL;

    if (length) {
        rand_string = (char*)malloc(sizeof(char) * (length + 1));

        if (rand_string) {            
            for (int n = 0; n < length; n++) {            
                int key = rand() % (int32_t)(sizeof(charset) - 1);
                rand_string[n] = charset[key];
            }

            rand_string[length] = '\0';
        }
    }

    return rand_string;
}


bool hash_sha256(void* input, size_t length, unsigned char* buf) {
    SHA256_CTX ctx;
    if(!SHA256_Init(&ctx))
        return false;

    if(!SHA256_Update(&ctx, (unsigned char*)input, length))
        return false;

    if(!SHA256_Final(buf, &ctx))
        return false;

    return true;
}

client_t* mariadb_get_all_clients(db_driver_t* db) {
    // TODO end this later;
}

// Remember: database lock must be acquired by the calling function
bool mariadb_insert_token(uint64_t client_id, char* token, db_driver_t* db) {
    if(strlen(token) > SHA256_DIGEST_LENGTH) {
        return false;
    }
    char* query = "INSERT INTO tokens (user_id, hashed_token, expiration_date) values (?, ?, ?)";

    MYSQL_STMT* stmt = mysql_stmt_init((MYSQL*)db->conn);
    if (mysql_stmt_prepare(stmt, query, strlen(query)+1) != 0) {
        fprintf(stderr, "Error preparing \"INSERT TOKEN\"statement: %s\n", mysql_stmt_error(stmt));
        return false;
    }

    MYSQL_TIME expiration_dt;
    time_t timestamp;
    struct tm* tm;

    time(&timestamp);
    tm = gmtime(&timestamp);
    tm->tm_mday += 1;
    mktime(tm);
 
    expiration_dt.year = tm->tm_year;
    expiration_dt.month = tm->tm_mon;
    expiration_dt.day = tm->tm_mday;
    expiration_dt.hour = tm->tm_hour;
    expiration_dt.minute = tm->tm_min;
    expiration_dt.second = tm->tm_sec;


    MYSQL_BIND args[3];
    memset(args, 0, sizeof(args));
    args[0].buffer_type = MYSQL_TYPE_LONGLONG;
    args[0].buffer = &client_id;
    args[0].is_unsigned = 1;

    args[1].buffer_type = MYSQL_TYPE_BIT;
    args[1].buffer = token;
    args[1].buffer_length = strlen(token)+1;

    args[2].buffer_type = MYSQL_TYPE_DATETIME;
    args[2].buffer = &expiration_dt;
    args[2].is_null = 0; // Not null
    mysql_stmt_bind_param(stmt, args);

    if (mysql_stmt_execute(stmt) != 0) {
        fprintf(stderr, "Error executing \"INSERT TOKEN\" statement: %s\n", mysql_stmt_error(stmt));
        return NULL;
    }
    mysql_stmt_close(stmt);
}

char* mariadb_login(db_driver_t* db, char* key, char* password) {
    //printf("\"%s\"\n", key);
    //printf("\"%s\"\n", password);
    uv_mutex_lock(&db->lock);
    char* query = "SELECT client_id FROM credentials where `client_key` = ? and `password` = ?";
    MYSQL_STMT* stmt = mysql_stmt_init((MYSQL*)db->conn);
    if (mysql_stmt_prepare(stmt, query, strlen(query)+1) != 0) {
        fprintf(stderr, "Error preparing statement: %s\n", mysql_stmt_error(stmt));
        uv_mutex_unlock(&db->lock);
        return NULL;
    }
    
    MYSQL_BIND args[2];
    memset(args, 0, sizeof(args));
    args[0].buffer_type = MYSQL_TYPE_STRING;
    args[0].buffer = key;
    args[0].buffer_length = strlen(key)+1;

    args[1].buffer_type = MYSQL_TYPE_STRING;
    args[1].buffer = password;
    args[1].buffer_length = strlen(password)+1;
    mysql_stmt_bind_param(stmt, args);
    if (mysql_stmt_execute(stmt) != 0) {
        fprintf(stderr, "Error executing statement: %s\n", mysql_stmt_error(stmt));
        mysql_stmt_close(stmt);
        uv_mutex_unlock(&db->lock);
        return NULL;
    }
    if (mysql_stmt_store_result(stmt) != 0) {
        fprintf(stderr, "Error storing result set: %s\n", mysql_stmt_error(stmt));
        mysql_stmt_close(stmt);
        uv_mutex_unlock(&db->lock);
        return NULL;
    }

    MYSQL_BIND result;
    uint64_t client_id;

    result.buffer_type = MYSQL_TYPE_LONG;
    result.buffer = &client_id;
    result.is_null = 0; // Not null

    char* token = (char*)malloc(SHA256_DIGEST_LENGTH);
    if(mysql_stmt_num_rows(stmt) != 0) {
        
        if(mysql_stmt_bind_result(stmt, &result) != 0) {
            fprintf(stderr, "Result binding failed: %s\n", mysql_stmt_error(stmt));
            mysql_stmt_close(stmt);
            uv_mutex_unlock(&db->lock);
            return NULL;
        }

        hash_sha256(rand_string(TOKEN_SIZE), TOKEN_SIZE, (unsigned char*)token);
        if(mariadb_insert_token(client_id, token, db)) {
            fprintf(stderr, "Inserting token failed: %s\n", mysql_stmt_error(stmt));
            mysql_stmt_close(stmt);
            uv_mutex_unlock(&db->lock);
            return NULL;
        }

        printf("User logged, returning him new token");

    } else {
        printf("User provided wrong credentials!");
    }

    mysql_stmt_close(stmt);
    uv_mutex_unlock(&db->lock);

    return token;
}

// TODO add a pool of connections, instead of using a single one
int32_t init_database(db_driver_ctx db_ctx, db_driver_t* db) {
    int32_t result;
    db->database_ctx = db_ctx;
    uv_mutex_init(&db->lock);
    switch(db_ctx.db_type) {
        case MARIADB: {
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
}

void destroy_database(db_driver_t* db) {
    switch (db->database_ctx.db_type) {
        case MARIADB:
            mysql_close((MYSQL*)db->conn);
            break;
    }
    uv_mutex_destroy(&db->lock);
}