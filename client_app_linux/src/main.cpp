#include <cstdlib>
#include <stdint.h>
#include <stdlib.h>
#include <sys/types.h>
#include <unistd.h>
#include <locale.h>
#include <string.h>
#include <stdio.h>

#include <uv.h>

#include "../includes/utils.h"
#include "../includes/lemon_ctx.h"
#include "../includes/lemon_gui.h"
#include "../includes/server_tcp.h"

#include "../../libs/imgui/imgui.h"
#include "../../libs/imgui/backends/imgui_impl_glfw.h"
#include "../../libs/imgui/backends/imgui_impl_opengl3.h"

#include <uv/unix.h>
#define GL_SILENCE_DEPRECATION
#include <GLFW/glfw3.h>

static void glfw_error_callback(int error, const char* description)
{
    fprintf(stderr, "GLFW Error %d: %s\n", error, description);
}

i32 main(int argc, char** argv) {

    srand(time(NULL));

    glfwSetErrorCallback(glfw_error_callback);
    if (!glfwInit())
        return EXIT_FAILURE;

    const char* glsl_version = "#version 130";
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 0);
    //glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);  // 3.2+ only
    //glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);            // 3.0+ only

    GLFWwindow* window = glfwCreateWindow(1280, 720, "Lemon comm", nullptr, nullptr);
    if (window == nullptr)
        return EXIT_FAILURE;
    glfwMakeContextCurrent(window);
    glfwSwapInterval(1); // Enable vsync | TODO change maybe later

    // Setup Dear ImGui context
    IMGUI_CHECKVERSION();
    ImGui::CreateContext();
    ImGuiIO& io = ImGui::GetIO(); (void)io;
    io.ConfigFlags |= ImGuiConfigFlags_NavEnableKeyboard;     // Enable Keyboard Controls
    io.ConfigFlags |= ImGuiConfigFlags_NavEnableGamepad;      // Enable Gamepad Controls

    // Setup Dear ImGui style
    ImGui::StyleColorsDark();
    //ImGui::StyleColorsLight();

    // Setup Platform/Renderer backends
    ImGui_ImplGlfw_InitForOpenGL(window, true);
    ImGui_ImplOpenGL3_Init(glsl_version);


    lemon_ctx ctx;
    init_lemon_ctx(argc, argv, &ctx, window);
    uv_loop_t* loop = uv_default_loop();
    ctx.loop = loop;

    uv_idle_t gui_handle;
    uv_idle_init(loop, &gui_handle);
    gui_handle.data = &ctx;
    uv_idle_start(&gui_handle, ui_main);

    uv_tcp_t server_handle;
    uv_tcp_init(loop, &server_handle);
    server_handle.data = &ctx;

    uv_work_t tcp_req;
    tcp_req.data = &ctx;
    uv_queue_work(loop, &tcp_req, tcp_loop, NULL);
    uv_run(loop, UV_RUN_DEFAULT);
    
    uv_loop_close(loop);

    ImGui_ImplOpenGL3_Shutdown();
    ImGui_ImplGlfw_Shutdown();
    ImGui::DestroyContext();
    destroy_lemon_ctx(&ctx);
    glfwTerminate();

    return 0;
}