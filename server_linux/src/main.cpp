#include <cstdlib>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include "../includes/server.h"
#include "../includes/requests.hpp"

#define __EXTRA_INFO
// TODO move checking for required arguments to function
// initing database
int32_t init_server(server_ctx* server_ctx, int32_t argc, char** argv) {
    if(argc < 2) {
        fprintf(stderr, "To few arguments!\n");
        return -1;
    }
    char* host, *user, *password, *db_name;
    host = user = password = db_name = NULL;

    uint16_t db_port = 0;
    server_ctx->port = 0;

    for(int32_t i = 1; i < argc; i++) {
#ifdef __EXTRA_INFO
        printf("Current parsed argument: \"%s\"\n", argv[i]);
#endif
        if(strcmp(SERVER_PORT_ARG, argv[i]) == 0) {
            if(argc == i+1) {
                fprintf(stderr, "\"%s\" requires a positional argument!\n", SERVER_PORT_ARG);
                return -1;
            }
            if((server_ctx->port = (uint16_t)atoi(argv[i+1])) == 0) {
                fprintf(stderr, "Cannot parse server port %s\n", argv[i+1]);
                return -1;
            }
            i++;
            continue;
        }
        if(strcmp(DATABASE_HOST_ARG, argv[i]) == 0) {
            if(argc == i+1) {
                fprintf(stderr, "\"%s\" requires a positional argument!\n", SERVER_PORT_ARG);
                return -1;
            }
            host = argv[i+1];
            i++;
            continue;
        }
        if(strcmp(DATABASE_USER_ARG, argv[i]) == 0) {
            if(argc == i+1) {
                fprintf(stderr, "\"%s\" requires a positional argument!\n", SERVER_PORT_ARG);
                return -1;
            }
            user = argv[i+1];
            i++;
            continue;
        }
        if(strcmp(DATABASE_PASSWORD_ARG, argv[i]) == 0) {
            if(argc == i+1) {
                fprintf(stderr, "\"%s\" requires a positional argument!\n", SERVER_PORT_ARG);
                return -1;
            }
            password = argv[i+1];
            i++;
            continue;
        }
        if(strcmp(DATABASE_PORT_ARG, argv[i]) == 0) {
            if(argc == i+1) {
                fprintf(stderr, "\"%s\" requires a positional argument!\n", SERVER_PORT_ARG);
                return -1;
            }
            if((db_port = (uint16_t)atoi(argv[i+1])) == 0) {
                fprintf(stderr, "Cannot parse database port %s\n", argv[i+1]);
                return -1;
            }
            i++;
            continue;
        }
        if(strcmp(DATABASE_NAME_ARG, argv[i]) == 0) {
            if(argc == i+1) {
                fprintf(stderr, "\"%s\" requires a positional argument!\n", SERVER_PORT_ARG);
                return -1;
            }
            db_name = argv[i+1];
            i++;
            continue;
        }

        printf("Unknown argument \"%s\"\n", argv[i]);
    }

    if(server_ctx->port == 0) {
        printf("Default server port is set: \"%d\"\n", DEFAULT_SERVER_PORT);
        server_ctx->port = DEFAULT_SERVER_PORT;
    }
    if(!user) {
        fprintf(stderr, "No user for database provided\n");
        return -1;
    }
    if(!password) {
        printf("Default password is set: \"%s\"\n", DEFAULT_DATABASE_PASSWORD);
        password = DEFAULT_DATABASE_PASSWORD;
    }
    if(!host) {
        printf("Default database host is set: \"%s\"\n", DEFAULT_DATABASE_HOST);
        host = DEFAULT_DATABASE_HOST;
    }
    if(!db_name) {
        printf("Default database is set: \"%s\"\n", DEFAULT_DATABASE_NAME);
        db_name = DEFAULT_DATABASE_NAME;
    }
    if(db_port == 0) {
        fprintf(stderr, "No database port provided\n");
        return -1;
    }

    db_driver_ctx db_ctx = {
        .db_type = MARIADB,
        .host = host,
        .user = user,
        .password = password,
        .port = db_port,
        .db_name = db_name,
        .options = 0,
    };

    server_ctx->database = (db_driver_t*)malloc(sizeof(db_driver_t));
    if(init_database(db_ctx, server_ctx->database) != 0) 
        return -1;
    
    init_client_list(&server_ctx->client_list);

    return 0;
}

char* get_client_ip(uv_tcp_t* client) {
    struct sockaddr_in* ipv4_addr;
    int namelen = sizeof(&ipv4_addr);
    uv_tcp_getpeername(client, (struct sockaddr*)&ipv4_addr, &namelen);
    char* ip_str = (char*)malloc(INET_ADDRSTRLEN + 1);
    uv_ip4_name(ipv4_addr, ip_str, INET_ADDRSTRLEN);
    return ip_str;
}

void destroy_ctx(server_ctx* ctx) {
    destroy_database(ctx->database);
    destroy_client_list(&ctx->client_list);
    free(ctx->database);
    //nothing to see here
}

void alloc_data(uv_handle_t* handle, size_t suggested_size, uv_buf_t* buf) {
    buf->base = (char*)malloc(suggested_size);
    buf->len = suggested_size;
}

typedef struct {
    client_t* client;
    server_ctx* ctx;
} client_conn_t;

void serve_client(uv_stream_t* client_stream, ssize_t nread, const uv_buf_t* buf) {
    
    client_conn_t* cc = (client_conn_t *)client_stream->data;
    client_list_t* cl = &cc->ctx->client_list;

    client_t* client = cc->client;
    uv_mutex_lock(&client->lock);

    if (nread > 0) {
        message_t* m = (message_t*)malloc(sizeof(message_t));
        m->err = NULL;
        deserialize_message(m, buf->base);
#ifdef __EXTRA_INFO
        printf("Got new message");
        printf("Whole message: \n%s\n", buf->base);
#endif
        switch(m->type) {
            case PARSE_ERR:
                // Inform client about wrong request
                basic_res(client_stream, ERR, "Wrong request");
                break;
            case RESPONSE:
                // Client should not be one sending responses
                // Just ping it back
                ping_back(client_stream, m);
                break;
            case LOGIN:
                login_user(cc->ctx, client, m);
                break;
            }
        destroy_message(m);
    } else if(nread == 0) {
        // No data was sent to the server
        basic_res(client_stream, ERR, "Wrong request");
    } else {
        // EOF file reached - closing connection
        printf("Client disconected\n");
        uv_rwlock_wrlock(&cl->lock);
        int32_t index;
        vec_remove(&cl->vec, client);
        uv_rwlock_rdunlock(&cl->lock);
    }

    // TODO lock still can be obtained while destroying client here - need fix
    uv_mutex_unlock(&client->lock);
    destroy_client(client);
    free(client);
    free(buf->base);
}

void on_new_connection(uv_stream_t *server, int status) {
    server_ctx* ctx = (server_ctx*)server->data;

    printf("\nNew connection!\n");
    if (status < 0) {
        fprintf(stderr, "New connection error %s\n", uv_strerror(status));
        return;
    }
    uv_tcp_t *client_stream = (uv_tcp_t*)malloc(sizeof(uv_tcp_t));
    uv_tcp_init(ctx->loop, client_stream);

#ifdef __EXTRA_INFO
    printf("Initialized client struct, creating a connection struct\n");
#endif

#ifdef __EXTRA_INFO
    printf("Created connection struct, accepting connection\n");
#endif

    if (uv_accept(server, (uv_stream_t*) client_stream) == 0) {

        // Init client struct
        client_t* client = (client_t*)malloc(sizeof(client_t));
        init_client(client,client_stream);

        // Init client connection struct
        client_conn_t* cc = (client_conn_t*)malloc(sizeof(client_conn_t));
        cc->client = client;
        cc->ctx = ctx;
        // Add cc to handle
        client_stream->data = cc;
        uv_read_start((uv_stream_t*) client_stream, alloc_data, serve_client);
    } else {
        fprintf(stderr, "Could not accept an incoming connection...\n");
        // Check if client_stream needs to be freed or not
        free(client_stream);
    }
}


int main(int argc, char** argv) {
    // Main loop / server configuration
    server_ctx ctx;
    int32_t result = init_server(&ctx, argc, argv);

    if(result != 0) {
        fprintf(stderr, "Cannot init server!\n");
        return -1;
    }

    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_port = htons(ctx.port)
    };

    uv_loop_t* loop = uv_default_loop();
    ctx.loop = loop;

    uv_tcp_t server;
    uv_tcp_init(loop, &server);
    uv_ip4_addr("0.0.0.0", ctx.port, &addr);
    uv_tcp_bind(&server, (const struct sockaddr*)&addr, 0);

    // Set options and data
    // TODO create an cli argument for delay
    uv_tcp_keepalive(&server, 1, 900);
    server.data = &ctx;

    int r = uv_listen((uv_stream_t*) &server, 10, on_new_connection);
    if (r) {
        fprintf(stderr, "Listen error %s\n", uv_strerror(r));
        return -1;
    }
    printf("Server listens on port: %d\n", ctx.port);
    uv_run(loop, UV_RUN_DEFAULT);
    uv_loop_close(loop);

    // TODO call uv_close on tcp handle
    destroy_ctx(&ctx);
    return 0;
}