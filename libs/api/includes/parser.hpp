#ifndef __parser
#define __parser

#include <netinet/in.h>
#include <stdint.h>
#include <sys/socket.h>
#include "../../vec/src/vec.h"

#define ROOT_NODE "root"
#define TYPE_NODE "type"
#define STATUS_NODE "status"
#define ERROR_NODE "error"

#define TOKEN_BUFFER_SIZE 4096
#define UNSHASHED_TOKEN_SIZE 32

// Since utf-8, need to multiply by 4
#define FIRST_NAME_LEN 50*4
#define LAST_NAME_LEN 50*4
#define EMAIL_LEN 256*4
#define PASSWORD_LEN 50*4
#define IPV4_LEN 15

#ifdef __cplusplus
#   define EXTERNC extern "C"
#else
#   define EXTERNC
#endif

// REMEMBER - when adding types here changes also need to be made in parser itself!!!
// if there is "RETURN" in type's name, this means only a server can produce this message
typedef enum {
    RESPONSE, // Basic response without body
    PARSE_ERR,
    LOGIN,
    LOGIN_RETURN,
    NETWORK_STATE,
    NETWORK_STATE_RETURN
} message_type;

// TODO move parse error from type to status
// cannot properly destroy the message if original type is unknown
static const char* MESSAGE_TYPE_NAME[] = {
    "response",
    "parse_err",
    "login",
    "login_return",
    "network_state",
    "network_state_return"
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
} token_data_t;
#define TOKEN_NODE "token"

typedef token_data_t login_r_data_t;

typedef struct {
    char first_name[FIRST_NAME_LEN];
    char last_name[LAST_NAME_LEN];
    char email[EMAIL_LEN];
    struct sockaddr_in ip;
} client_body_t;

#define CLIENTS_NODE "clients"
#define SINGLE_CLIENT_NODE "client"

#define FIRST_NAME_NODE "first_name"
#define LAST_NAME_NODE "last_name"
#define EMAIL_NODE "email"
#define IPV4_NODE "ipv4"
#define PORT_NODE "port"

typedef vec_t(client_body_t) client_body_vec_t;

typedef struct {
    client_body_vec_t clients;
} network_state_return_data_t;

typedef token_data_t network_state_data_t;

typedef struct {
    message_type type;
    message_status status;
    char* err;

    union {
        login_data_t login;
        login_r_data_t login_r;
        network_state_data_t network;
        network_state_return_data_t network_r;
    } data;
} message_t;

EXTERNC void init_message(message_t* m, message_type, message_status status, const char* err);
EXTERNC void destroy_message(message_t* m);

EXTERNC char* serialize_message(message_t* m);
void deserialize_message(message_t* message, const char* raw_xml);

#endif
