#ifndef __LEMON_CTX
#define __LEMON_CTX

#include <GLFW/glfw3.h>
#include <stdbool.h> 

#include <uv.h>
#include "utils.h"

#include "../imgui/imgui.h"


#define NAME_LENGHT 32
#define PASSWORD_LENGHT 32

typedef struct lemon_client_ctx {
    uv_rwlock_t lock;
    char name[NAME_LENGHT];
    char password[PASSWORD_LENGHT];
} lemon_client_ctx;

typedef struct lemon_gui_ctx {
    GLFWwindow* window;
    // --- Gui variables ---

    // Universal
    bool ifDemoWindow;
    
    char temp_name[NAME_LENGHT];
    char temp_password[PASSWORD_LENGHT];

} lemon_gui_ctx;


typedef struct lemon_app_ctx {
    lemon_gui_ctx* gui_ctx;
    lemon_client_ctx* client_ctx;
    u64 flags;
} lemon_app_ctx;

lemon_app_ctx * init_app_ctx(i32 argc, char ** argv, GLFWwindow* window);
void free_app_context(lemon_app_ctx * app_ctx);

#endif