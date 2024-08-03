#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/types.h>
#include <unistd.h>
#include <locale.h>
#include <string.h> 
#include <GLFW/glfw3.h>
#include <argp.h>


#include <uv.h>
#include <uv/unix.h>
#include "../includes/utils.h"
#include "../includes/lemon_ctx.h"

void init_lemon_ctx(int32_t argc, char** argv, lemon_ctx* ctx, GLFWwindow* window) {

    // TODO make argument parser
    for(int i = 1; i < argc; i++) {
        if(strcmp(argv[i], "--TEST")) {
            printf("---ARG TEST---\n");
        }
    }

    uv_rwlock_init(&ctx->client_ctx.lock);

    uv_rwlock_init(&ctx->tcp.connection_state_lock);
    uv_mutex_init(&ctx->tcp.request_lock);
    uv_mutex_init(&ctx->tcp.token_lock);
    
    ctx->tcp.conn_state = NO_CONNECTION;

    vec_init(&ctx->tcp.requests);

    ctx->gui.window = window;
    ctx->gui.if_demo_window = false;

    // Flags
    ctx->flags = 0;
}

void destroy_lemon_ctx(lemon_ctx * ctx) {
    glfwDestroyWindow(ctx->gui.window);

    vec_deinit(&ctx->tcp.requests);

    uv_rwlock_destroy(&ctx->client_ctx.lock);
    uv_rwlock_destroy(&ctx->tcp.connection_state_lock);
    uv_mutex_destroy(&ctx->tcp.request_lock);
    uv_mutex_destroy(&ctx->tcp.token_lock);
}

void push_request(lemon_ctx *ctx, tcp_request_t *req) {
    uv_mutex_lock(&ctx->tcp.request_lock);
    vec_push(&ctx->tcp.requests, req);
    uv_mutex_unlock(&ctx->tcp.request_lock);
}

void change_connection_state(lemon_ctx* ctx, connection_state new_state) {
    uv_rwlock_wrlock(&ctx->tcp.connection_state_lock);
    ctx->tcp.conn_state = new_state;
    uv_rwlock_wrunlock(&ctx->tcp.connection_state_lock);
}
connection_state check_connection_state(lemon_ctx* ctx) {
    uv_rwlock_rdlock(&ctx->tcp.connection_state_lock);
    connection_state state = ctx->tcp.conn_state;
    uv_rwlock_rdunlock(&ctx->tcp.connection_state_lock);
    return state;
}

void init_tcp_request(tcp_request_t *req) {
    memset(req, 0, sizeof(tcp_request_t));
    req->status = IDLE;
    uv_mutex_init(&req->lock);
}

void destroy_tcp_request(tcp_request_t *req) {
    uv_mutex_destroy(&req->lock);
}