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

#define TM_GREATER 1
#define TM_EQUAL 0
#define TM_SMALLER -1


struct tm* get_current_tm() {
    time_t timestamp;
    struct tm* tm;

    time(&timestamp);
    return gmtime(&timestamp);
}

void mysql_to_tm(MYSQL_TIME* mt, struct tm* tm) {
    tm->tm_year = mt->year;
    tm->tm_mon = mt->month;
    tm->tm_mday = mt->day;
    tm->tm_hour = mt->hour;
    tm->tm_min = mt->minute;
    tm->tm_sec = mt->second;
};

void tm_to_mysql(struct tm* tm, MYSQL_TIME* mt) {
    mt->year = tm->tm_year;
    mt->month = tm->tm_mon;
    mt->day = tm->tm_mday;
    mt->hour = tm->tm_hour;
    mt->minute = tm->tm_min;
    mt->second = tm->tm_sec;
}

// Compares tm1 to tm2
int8_t compare_tm(struct tm* tm1, struct tm* tm2) {
    // Linus Torvalds wouldn't be happy
    if(tm1->tm_year > tm2->tm_year)
        if(tm1->tm_mon > tm2->tm_mon)
            if(tm1->tm_mday > tm2->tm_mday)
                if(tm1->tm_hour > tm2->tm_hour)
                    if(tm1->tm_min > tm2->tm_min)
                        if(tm1->tm_sec > tm2->tm_sec)
                            return TM_GREATER;

    if(tm1->tm_year > tm2->tm_year)
        if(tm1->tm_mon > tm2->tm_mon)
            if(tm1->tm_mday > tm2->tm_mday)
                if(tm1->tm_hour > tm2->tm_hour)
                    if(tm1->tm_min > tm2->tm_min)
                        if(tm1->tm_sec > tm2->tm_sec)
                            return TM_SMALLER;
                    
    return TM_EQUAL;
}

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
bool mariadb_insert_token(db_driver_t* db, uint64_t client_id, char* token) {
    if(strlen(token) > SHA256_DIGEST_LENGTH) {
        return false;
    }
    char* query = "INSERT INTO tokens (client_id, hashed_token, expiration_date) values (?, ?, ?)";

    MYSQL_STMT* stmt = mysql_stmt_init((MYSQL*)db->conn);
    if (mysql_stmt_prepare(stmt, query, strlen(query)+1) != 0) {
        fprintf(stderr, "Error preparing \"INSERT TOKEN\"statement: %s\n", mysql_stmt_error(stmt));
        return false;
    }

    MYSQL_TIME expiration_dt;
    struct tm* tm;
    tm = get_current_tm();

    tm->tm_mday += 1;
    mktime(tm);
 
    tm_to_mysql(tm, &expiration_dt);

    MYSQL_BIND args[3];
    memset(args, 0, sizeof(args));
    args[0].buffer_type = MYSQL_TYPE_LONGLONG;
    args[0].buffer = &client_id;
    args[0].is_unsigned = 1;

    args[1].buffer_type = MYSQL_TYPE_STRING;
    args[1].buffer = token;
    args[1].buffer_length = strlen(token)+1;

    args[2].buffer_type = MYSQL_TYPE_DATETIME;
    args[2].buffer = &expiration_dt;
    args[2].is_null = 0;
    if(mysql_stmt_bind_param(stmt, args) != 0) {
        fprintf(stderr, "Error binding params in \"INSERT TOKEN\" statement: %s\n", mysql_stmt_error(stmt));
        mysql_stmt_close(stmt);
        return false;
    }

    if(mysql_stmt_execute(stmt) != 0) {
        fprintf(stderr, "Error executing \"INSERT TOKEN\" statement: %s\n", mysql_stmt_error(stmt));
        mysql_stmt_close(stmt);
        return false;
    }

    mysql_stmt_close(stmt);
    return true;
}

bool mariadb_check_token(db_driver_t* db, client_t* client, char* token) {
    uv_mutex_lock(&db->lock);

    char hashed_token[SHA256_DIGEST_LENGTH];
    hash_sha256(token, strlen(token)+1, (unsigned char*)hashed_token);

    char* query = "SELECT expiration_date from tokens where hashed_token = ? and client_id = ?";
    MYSQL_STMT* stmt = mysql_stmt_init((MYSQL*)db->conn);
    if (mysql_stmt_prepare(stmt, query, strlen(query)+1) != 0) {
        fprintf(stderr, "Error preparing \"CHECK TOKEN\"statement: %s\n", mysql_stmt_error(stmt));
        return false;
    }

    MYSQL_BIND args[2];
    memset(args, 0, sizeof(args));
    args[0].buffer_type = MYSQL_TYPE_STRING;
    args[0].buffer = &client->id;
    args[0].is_unsigned = 1;

    args[1].buffer_type = MYSQL_TYPE_LONGLONG;
    args[1].buffer = token;
    args[1].buffer_length = strlen(token)+1;

    if(mysql_stmt_bind_param(stmt, args) != 0) {
        fprintf(stderr, "Error binding params in \"CHECK TOKEN\" statement: %s\n", mysql_stmt_error(stmt));
        mysql_stmt_close(stmt);
        uv_mutex_unlock(&db->lock);
        return false;
    }

    if(mysql_stmt_execute(stmt) != 0) {
        fprintf(stderr, "Error executing \"CHECK TOKEN\" statement: %s\n", mysql_stmt_error(stmt));
        mysql_stmt_close(stmt);
        uv_mutex_unlock(&db->lock);
        return false;
    }

    MYSQL_BIND result;
    MYSQL_TIME* time;

    result.buffer_type = MYSQL_TYPE_DATETIME;
    result.buffer = time;
    result.is_null = 0;

    if(mysql_stmt_num_rows(stmt) != 0) {
        
        if(mysql_stmt_bind_result(stmt, &result) != 0) {
            fprintf(stderr, "Result binding failed: %s\n", mysql_stmt_error(stmt));
            mysql_stmt_close(stmt);
            uv_mutex_unlock(&db->lock);
            return false;
        }

        printf("Client provided good token.");
    } else {
        printf("Client provided wrong token!");
    }

    struct tm token_expiration_date;
    mysql_to_tm(time, &token_expiration_date);

    int8_t compare_result = compare_tm(&token_expiration_date, get_current_tm());

    if(compare_result == TM_SMALLER) {
        mysql_stmt_close(stmt);
        uv_mutex_unlock(&db->lock);
        return false;
    }

    mysql_stmt_close(stmt);
    uv_mutex_unlock(&db->lock);
    return true;
}

char* mariadb_login(db_driver_t* db, client_t client, char* key, char* password) {
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
        if(!mariadb_insert_token(db, client_id, token)) {
            fprintf(stderr, "Inserting token failed");
            mysql_stmt_close(stmt);
            uv_mutex_unlock(&db->lock);
            return NULL;
        }

        printf("User logged, returning him new token");

    } else {
        printf("User provided wrong credentials!");
    }


    // Init client fields after login
    // like ID
    client.id = client_id;

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