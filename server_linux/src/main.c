#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <uv.h>
#include <time.h>
#include "../includes/client.h"

typedef struct {
    uint16_t port;
    uv_loop_t* loop;
} server_ctx;


int8_t init_server(server_ctx* conf, int32_t argc, int8_t** argv) {
    if(argc < 2) {
        return -1;
    }
    int16_t port = atoi((char *)argv[1]);
    if(port != 0) {
        conf->port = port;
    }
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

void destroy_conf(server_ctx* conf) {
    //nothing to see here
}

void alloc_data(uv_handle_t* handle, size_t suggested_size, uv_buf_t* buf) {
    buf->base = (char*)malloc(suggested_size);
    buf->len = suggested_size;
}

void serve_client(uv_stream_t* client_stream, ssize_t nread, const uv_buf_t* buf) {
    client_t* client = (client_t *)client_stream->data;

    if(nread == UV_EOF || strcmp(buf->base, "\n") == 0 || strcmp(buf->base, "\0") == 0) {
        printf("Client disconected\n");
        destroy_client(client);
        free(buf->base);
    } else if (nread > 0) {
        // Process the received data (e.g., echo it back)
        uv_write_t* req = (uv_write_t*)malloc(sizeof(uv_write_t));
        uv_buf_t wrbuf = uv_buf_init(buf->base, buf->len);
        uv_write(req, client->stream, &wrbuf, 1, NULL);
        free(req);
    } else if (nread < 0) {
        printf("Client exploded?\n");
        destroy_client(client);
        free(buf->base);
    }
}

void on_new_connection(uv_stream_t *server, int status) {
    server_ctx* ctx = (server_ctx*)server->data;

    if (status < 0) {
        fprintf(stderr, "New connection error %s\n", uv_strerror(status));
        return;
    }
    printf("New connection!\n");

    uv_tcp_t *client_stream = (uv_tcp_t*) malloc(sizeof(uv_tcp_t));
    uv_tcp_init(ctx->loop, client_stream);
    client_t* client = init_client(client_stream);

    client_stream->data = client;

    if (uv_accept(server, (uv_stream_t*) client_stream) == 0) {
        uv_read_start((uv_stream_t*) client_stream, alloc_data, serve_client);
    } else {
        fprintf(stderr, "Could not accept an incoming connection...\n");
        destroy_client(client);
    }
}

void check_connections(uv_timer_t* timer) {

}

int main(int argc, char** argv) {
    // Main loop / server configuration
    server_ctx conf;
    int32_t result = init_server(&conf, argc, (signed char**)argv);

    if(result != 0) {
        fprintf(stderr, "Cannot init server!\n");
        return -1;
    }

    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_port = htons(conf.port)
    };

    uv_loop_t* loop = uv_default_loop();
    conf.loop = loop;

    uv_tcp_t server;
    uv_tcp_init(loop, &server);
    uv_ip4_addr("0.0.0.0", conf.port, &addr);
    uv_tcp_bind(&server, (const struct sockaddr*)&addr, 0);

    server.data = &conf;

    int r = uv_listen((uv_stream_t*) &server, 10, on_new_connection);
    if (r) {
        fprintf(stderr, "Listen error %s\n", uv_strerror(r));
        return -1;
    }
    printf("Server listens on port: %d\n", conf.port);

    // Timer loop configuration (used for disconnecting idle clients)
    uv_timer_t idle_checker;
    uv_timer_init(loop, &idle_checker);
    uv_timer_start(&idle_checker, check_connections, 10000, 1000);

    uv_run(loop, UV_RUN_DEFAULT);
    uv_loop_close(loop);

    // TODO call uv_close on tcp handle
    destroy_conf(&conf);
    return 0;
}