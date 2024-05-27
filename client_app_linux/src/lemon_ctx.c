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
#include "./includes/utils.h"
#include "./includes/lemon_ctx.h"

lemon_app_ctx * init_app_ctx(i32 argc, char ** argv, GLFWwindow* window) {
    lemon_client_ctx* client_ctx = (lemon_client_ctx*)malloc(sizeof(lemon_client_ctx));
    lemon_gui_ctx* gui_ctx = (lemon_gui_ctx*)malloc(sizeof(lemon_gui_ctx));
    lemon_app_ctx* app_ctx = (lemon_app_ctx*)malloc(sizeof(lemon_app_ctx));

    // client_ctx->name = NULL;
    // client_ctx->password = NULL;
    uv_rwlock_init(&client_ctx->lock);

    gui_ctx->window = window;
    gui_ctx->ifDemoWindow = false;

    // TODO make argument parser
    for(int i = 1; i < argc; i++) {
        if(strcmp(argv[i], "--TEST")) {
            printf("---ARG TEST---\n");
        }
    }

    app_ctx->client_ctx = client_ctx;
    app_ctx->gui_ctx = gui_ctx;
    app_ctx->flags = 0;
    return app_ctx;
}

void free_app_context(lemon_app_ctx * app_ctx) {
    glfwDestroyWindow(app_ctx->gui_ctx->window);
    free(app_ctx->client_ctx);
    free(app_ctx->gui_ctx);
    free(app_ctx);
}