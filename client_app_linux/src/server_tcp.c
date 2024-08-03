#include <netdb.h> 
#include <netinet/in.h> 
#include <stdlib.h> 
#include <string.h> 
#include <sys/socket.h> 
#include <sys/types.h> 
#include <unistd.h> 
#include <stdint.h>
#include <netinet/in.h>
#include <uv.h>
#include <uv/unix.h>
#include "../includes/lemon_ctx.h"
#include "../../libs/api/includes/parser.hpp"

#define MESSAGE_LEN 2097152 // 2 megabytes

// TODO need to change "serialize_message", so it also returns the lenght of the message
int32_t send_message(message_t* mes, int32_t sockfd) {
    char* ser_mes = serialize_message(mes);
    int32_t result = send(sockfd, ser_mes, sizeof(ser_mes), 0);
    free(ser_mes);
    return result;
}

void connect_to_server(tcp_request_t* req, lemon_ctx *ctx, int32_t sockfd) {
    lemon_tcp_ctx* tcp = &ctx->tcp;

    if(sockfd) {
        close(sockfd);
        change_connection_state(ctx, NO_CONNECTION);
    }

    sockfd = socket(AF_INET, SOCK_STREAM, 0); 
    if (sockfd == -1) { 
        change_connection_state(ctx, NO_CONNECTION);
        strcpy(req->err, "Cannot create the socket.");
        req->status = FINISHED_ERR;
        fprintf(stderr, "socket creation failed\n");
        return;
    } else {
        printf("Socket created\n");
    }

    // Connecting...
    change_connection_state(ctx, CONNECTING);
    if(!connect(sockfd, (struct sockaddr*)&tcp->dest, sizeof(tcp->dest))) {
        change_connection_state(ctx, CONNECTED);
        req->status = FINISHED_OK;
        printf("Connected to server\n");
    } else {
        change_connection_state(ctx, CANNOT_CONNECT);
        strcpy(req->err, "Cannot connect to the server.");
        req->status = FINISHED_ERR;
        fprintf(stderr, "Cannot connect to the server\n");
        return;
    }
}

void disconnect_from_server(tcp_request_t* req, lemon_ctx* ctx, int32_t sockfd) {
    lemon_tcp_ctx* tcp = &ctx->tcp;
    if(sockfd) {
        close(sockfd);
        change_connection_state(ctx, NO_CONNECTION);
    }
    req->status = FINISHED_OK;
}

void login(tcp_request_t* req, lemon_ctx* ctx, int32_t sockfd) {
    message_t mes;
    init_message(&mes, LOGIN, OK, NULL);

    uv_rwlock_rdlock(&ctx->client_ctx.lock);
    login_data_t data = {
        .key = ctx->client_ctx.key,
        .password = ctx->client_ctx.password
    };
    mes.data.login = data;
    uv_rwlock_rdunlock(&ctx->client_ctx.lock);
    
    int32_t result;
    result = send_message(&mes, sockfd);
    destroy_message(&mes);

    if(result == -1) {
        req->status = FINISHED_ERR;
        strcpy(req->err, "Cannot send the message.");
        fprintf(stderr, "Cannot send login message");
        return;
    }

    char buf[MESSAGE_LEN];
    result = read(sockfd, buf, MESSAGE_LEN);

    if(result == -1) {
        req->status = FINISHED_ERR;
        strcpy(req->err, "Cannot read the message.");
        fprintf(stderr, "Cannot read login message");
        return;
    }

    printf("%s\n", buf);

    req->status = FINISHED_OK;
}

void tcp_loop(uv_work_t* req) {
    lemon_ctx* ctx = (lemon_ctx*)req->data;
    lemon_tcp_ctx* tcp = &ctx->tcp;
    tcp_request_t* tcp_req;

    int sockfd = 0; 
    char buf[MESSAGE_LEN];

    while(1) {
        bzero(buf, MESSAGE_LEN);
        tcp_req = NULL;

        uv_mutex_lock(&tcp->request_lock);
        if(tcp->requests.length) {
            tcp_req = vec_first(&tcp->requests);
        }
        uv_mutex_unlock(&tcp->request_lock);
        if(tcp_req != NULL) {
            uv_mutex_lock(&tcp_req->lock);
            tcp_req->status = RUNNING;
            switch (tcp_req->action) {
                case CONNECT_REQUEST: {
                    connect_to_server(tcp_req, ctx, sockfd);
                    break;
                }
                case DISCONNECT_REQUEST: {
                    disconnect_from_server(tcp_req, ctx, sockfd);
                    break;
                }
                case LOGIN_REQUEST: {
                    login(tcp_req, ctx, sockfd);
                    break;
                }
            }

            uv_mutex_unlock(&tcp_req->lock);
            vec_remove(&tcp->requests, tcp_req);
        }
    }
}