#ifndef __LEMON_CTX
#define __LEMON_CTX

#include <netinet/in.h>
#include <stdbool.h>
#include <uv.h>
#include <GLFW/glfw3.h>
#include <uv/unix.h>
#include "../../libs/api/includes/parser.hpp"
#include "../../libs/vec/src/vec.h"
#include "../../libs/api/includes/database.h"
#include "utils.h"

typedef struct {
    uv_rwlock_t lock;
    char first_name[FIRST_NAME_LEN];
    char last_name[LAST_NAME_LEN];
    char* key; // Certificate
    char password[PASSWORD_LEN];
    char email[EMAIL_LEN];
} lemon_client_ctx;

// had to change "connection state" to "connection_state" - changes meaning of some other struct
typedef enum {
    NO_CONNECTION,
    CONNECTING,
    CONNECTED,
    CANNOT_CONNECT
} connection_state;

typedef enum {
    CONNECT_REQUEST,
    DISCONNECT_REQUEST, // Used also for simply reseting after failed connection.
    LOGIN_REQUEST,
} tcp_request_action;

typedef enum {
    IDLE,
    RUNNING,
    FINISHED_OK,
    FINISHED_ERR
} tcp_request_status;

typedef struct {
    uv_mutex_t lock;
    tcp_request_action action;
    tcp_request_status status;
    char err[4096];
    void* data;
} tcp_request_t;

void init_tcp_request(tcp_request_t* req);
void destroy_tcp_request(tcp_request_t* req);

typedef vec_t(tcp_request_t*) requests_vec_t;

typedef struct {
    uv_rwlock_t connection_state_lock;
    connection_state conn_state;
    struct sockaddr_in dest;

    uv_mutex_t request_lock;
    requests_vec_t requests;

    uv_mutex_t token_lock;
    token_t token;
} lemon_tcp_ctx;

typedef struct {
    GLFWwindow* window;
    // --- Gui variables ---

    // Universal
    bool if_demo_window;
} lemon_gui_ctx;

typedef struct {
    uv_loop_t* loop;
    lemon_gui_ctx gui;
    lemon_tcp_ctx tcp;
    lemon_client_ctx client_ctx;
    u64 flags;
} lemon_ctx;

void change_connection_state(lemon_ctx* ctx, connection_state new_state);
connection_state check_connection_state(lemon_ctx* ctx);

void init_request(tcp_request_t* req);
void push_request(lemon_ctx* ctx, tcp_request_t* req);

void init_lemon_ctx(i32 argc, char ** argv, lemon_ctx* ctx, GLFWwindow* window);
void destroy_lemon_ctx(lemon_ctx * app_ctx);

#endif