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

// TODO timestamps use may be redundant - find better solution
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

void generate_new_token(token_t* token) {
    char* new_token = rand_string(UNSHASHED_TOKEN_SIZE);

    memcpy(token->data, new_token, UNSHASHED_TOKEN_SIZE);
    token->is_hashed = false;
    token->is_null = false;
    token->length = UNSHASHED_TOKEN_SIZE;

    free(new_token);
}

void hash_token_sha256(token_t* token) {
    hash_sha256(token->data, token->length, (unsigned char*) token->data);
}

client_t* mariadb_get_all_clients(db_driver_t* db) {
    // TODO end this later;
}

// Remember: database lock must be acquired by the calling function
// TODO make custom struct for tokens to store their length
bool mariadb_insert_token(db_driver_t* db, uint64_t client_id, token_t* token) {
    // if(strlen(token) > SHA256_DIGEST_LENGTH) {
    //     fprintf(stderr, "Error while \"INSERT TOKEN\" statement: SHA256 token to long\n");
    //     return false;
    // }
    char* query = "INSERT INTO tokens (client_id, hashed_token, expiration_date) values (?, ?, ADDTIME(NOW(), '24:0:0'))";

    MYSQL_STMT* stmt = mysql_stmt_init((MYSQL*)db->conn);
    if (mysql_stmt_prepare(stmt, query, strlen(query)+1) != 0) {
        fprintf(stderr, "Error preparing \"INSERT TOKEN\" statement: %s\n", mysql_stmt_error(stmt));
        return false;
    }

    //MYSQL_TIME expiration_dt;
    //struct tm* tm;
    //tm = get_current_tm();

    //tm->tm_mday += 1;
    //mktime(tm);
 
    //tm_to_mysql(tm, &expiration_dt);

    MYSQL_BIND args[2];
    memset(args, 0, sizeof(args));
    args[0].buffer_type = MYSQL_TYPE_LONGLONG;
    args[0].buffer = &client_id;
    args[0].is_unsigned = 1;

    args[1].buffer_type = MYSQL_TYPE_STRING;
    args[1].buffer = token->data;
    args[1].buffer_length = TOKEN_BUFFER_SIZE;
    args[1].length = &token->length;

    //args[2].buffer_type = MYSQL_TYPE_DATETIME;
    //args[2].buffer = &expiration_dt;
    //args[2].is_null = 0;
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

bool mariadb_check_token(db_driver_t* db, client_t* client, token_t* token_to_check) {
    uv_mutex_lock(&db->lock);

    if(!token_to_check->is_hashed)
        hash_token_sha256(token_to_check);

    char* query = "SELECT hashed_token from tokens where client_id = ? and expiration_date > UTC_TIMESTAMP()";
    MYSQL_STMT* stmt = mysql_stmt_init((MYSQL*)db->conn);
    if (mysql_stmt_prepare(stmt, query, strlen(query)+1) != 0) {
        uv_mutex_unlock(&db->lock);
        fprintf(stderr, "Error preparing \"CHECK TOKEN\"statement: %s\n", mysql_stmt_error(stmt));
        return false;
    }

    MYSQL_BIND args[1];
    memset(args, 0, sizeof(args));
    args[0].buffer_type = MYSQL_TYPE_LONGLONG;
    args[0].buffer = &client->id;
    args[0].is_unsigned = 1;

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

    if (mysql_stmt_store_result(stmt) != 0) {
        fprintf(stderr, "Error storing results in \"CHECK TOKEN\" statement: %s\n", mysql_stmt_error(stmt));
        mysql_stmt_close(stmt);
        uv_mutex_unlock(&db->lock);
        return NULL;
    }

    MYSQL_BIND result;
    memset(&result, 0, sizeof(result));

    token_t returned_token;
    memset(&returned_token, 0, sizeof(token_t));

    int64_t i = 0;
    result.buffer_type = MYSQL_TYPE_STRING;
    result.buffer_length = TOKEN_BUFFER_SIZE;
    result.buffer = returned_token.data;
    result.length = &returned_token.length;
    result.is_null = (char*)&returned_token.is_null;

    if(mysql_stmt_bind_result(stmt, &result) != 0) {
        fprintf(stderr, "Error binding results in \"CHECK TOKEN\" statement: %s\n", mysql_stmt_error(stmt));
        mysql_stmt_close(stmt);
        uv_mutex_unlock(&db->lock);
        return false;
    }
    
    bool flag1 = false;
    int32_t fetch_result = mysql_stmt_fetch(stmt);
    while (fetch_result == 0 || fetch_result == MYSQL_DATA_TRUNCATED) {
        bool flag2 = true;
        for(int8_t i = 0; i < returned_token.length; i++) {
            if(returned_token.data[i] != token_to_check->data[i]) {
                flag2 = false;
                break;
            }
        }

        if(flag2) {
            flag1 = true;
            break;
        }

        fetch_result = mysql_stmt_fetch(stmt);
    }

    mysql_stmt_close(stmt);
    uv_mutex_unlock(&db->lock);
    return flag1;
}

token_t* mariadb_login(db_driver_t* db, client_t* client, char* key, char* password) {
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

    if (mysql_stmt_bind_param(stmt, args) != 0) {
        fprintf(stderr, "Error binding params statement: %s\n", mysql_stmt_error(stmt));
        mysql_stmt_close(stmt);
        uv_mutex_unlock(&db->lock);
        return NULL;
    }
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
    memset(&result, 0, sizeof(result));
    uint64_t client_id = 250;

    result.buffer_type = MYSQL_TYPE_LONG;
    result.buffer = &client_id;
    result.is_null = 0; // Not null

    token_t* token = (token_t*)malloc(sizeof(token_t));
    generate_new_token(token);
    token_t hashed_token = *token;
    hash_token_sha256(&hashed_token);

    if(mysql_stmt_bind_result(stmt, &result) != 0) {
        fprintf(stderr, "Result binding failed: %s\n", mysql_stmt_error(stmt));
        mysql_stmt_close(stmt);
        uv_mutex_unlock(&db->lock);
        return NULL;
    }

    if(mysql_stmt_num_rows(stmt) != 0) {
        if(mysql_stmt_fetch(stmt)) {
            fprintf(stderr, "Result fetching failed: %s\n", mysql_stmt_error(stmt));
            mysql_stmt_close(stmt);
            uv_mutex_unlock(&db->lock);
            return NULL;
        }
        if(!mariadb_insert_token(db, client_id, &hashed_token)) {
            fprintf(stderr, "Inserting token failed\n");
            mysql_stmt_close(stmt);
            uv_mutex_unlock(&db->lock);
            return NULL;
        }

        printf("User logged, returning him new token\n");

    } else {
        printf("User provided wrong credentials!\n");
    }

    //printf("NEW UNHASHED TOKEN: \"%s\"\n", unhashed_token);
    //printf("%s\n", token);


    // Init client fields after login
    // like ID
    client->id = client_id;

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
            db->check_token = mariadb_check_token;
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