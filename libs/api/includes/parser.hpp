#include <stdint.h>

#define NAME_LENGHT 32
#define PASSWORD_LENGHT 32

#define TYPE_NODE "type"
#define STATUS_NODE "status"
#define ERROR_NODE "error"

struct client {
    int8_t name[NAME_LENGHT];
    int8_t password[NAME_LENGHT];
};

typedef enum message_type {
    RESPONSE, // Basic response without body.
    USER_REGISTRATION
} message_type;

static const char* MESSAGE_TYPE_NAME[] = {
    "response",
    "user_registration"
};

typedef enum message_status {
    OK,
    ERR
} message_status;

static const char* MESSAGE_STATUS_NAME[] = {
    "ok",
    "err"
};

typedef struct message {
    message_type type;
    message_status status;
    char* err;

    void* data;
} message;

typedef struct register_user_data {
    const char* name[NAME_LENGHT];
    const char* password[PASSWORD_LENGHT];
} register_user_data;

message create_response(message_status status, char* err);
void destroy_response(message* m);

char* deserialize_response(message* response);
bool serialize_response(message* message, const char* raw_xml);
