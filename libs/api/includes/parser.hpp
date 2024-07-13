#ifndef __parser
#define __parser

#include <stdint.h>

#define TYPE_NODE "type"
#define STATUS_NODE "status"
#define ERROR_NODE "error"

typedef enum {
    RESPONSE, // Basic response without body
    LOGIN,
    LOGIN_RETURN,
    PARSE_ERR,
} message_type;

static const char* MESSAGE_TYPE_NAME[] = {
    "response",
    "parse_err",
    "login",
    "login_return",
};

typedef enum {
    OK,
    ERR,
} message_status;

static const char* MESSAGE_STATUS_NAME[] = {
    "ok",
    "err"
};

typedef struct {
    char* key;
    char* password;
} login_data_t;

#define KEY_NODE "key"
#define PASSWORD_NODE "password"

// Login return data
typedef struct {
    char* token;
} login_r_data_t;

#define TOKEN_NODE "token"

typedef struct {
    message_type type;
    message_status status;
    char* err;

    union {
        login_data_t login;
        login_r_data_t login_r;
    } data;
} message_t;

void init_message(message_t* m, message_type, message_status status, const char* err);
void destroy_message(message_t* m);

char* serialize_message(message_t* m);
void deserialize_message(message_t* message, const char* raw_xml);

#endif
