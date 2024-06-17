#ifndef __parser
#define __parser

#include <stdint.h>

#define TYPE_NODE "type"
#define STATUS_NODE "status"
#define ERROR_NODE "error"

typedef enum {
    RESPONSE, // Basic response without body
    LOGIN,
    PARSE_ERR
} message_type;

static const char* MESSAGE_TYPE_NAME[] = {
    "response",
    "login"
};

typedef enum {
    OK,
    ERR
} message_status;

static const char* MESSAGE_STATUS_NAME[] = {
    "ok",
    "err"
};

typedef struct {
    char* key;
    char* password;
} login_data_t;

typedef struct {
    message_type type;
    message_status status;
    char* err;

    union {
        login_data_t login;
    } data;
} message_t;

void init_message(message_t* m, message_status status, const char* err);
void destroy_message(message_t* m);

char* serialize_message(message_t* m);
void deserialize_message(message_t* message, const char* raw_xml);

#endif
