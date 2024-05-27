#include <cstdlib>
#include <stdint.h>
#include <stdlib.h>
#include <sys/types.h>
#include <unistd.h>
#include <locale.h>
#include <string.h>
#include <stdio.h>

#include <uv.h>
#include "includes/utils.h"

#include "../imgui/imgui.h"
#include "../imgui/backends/imgui_impl_glfw.h"
#include "../imgui/backends/imgui_impl_opengl3.h"

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

    lemon_app_ctx* lemon_app_ctx = init_app_ctx(argc, argv, window);

    uv_loop_t* gui_loop = uv_default_loop();
    uv_idle_t gui_handle;
    uv_idle_init(gui_loop, &gui_handle);
    gui_handle.data = lemon_app_ctx;
    uv_idle_start(&gui_handle, ui_main);

    uv_run(gui_loop, UV_RUN_DEFAULT);
    uv_loop_close(gui_loop);

    fprintf(stdout, "OKNO się zamknęło\n");

    ImGui_ImplOpenGL3_Shutdown();
    ImGui_ImplGlfw_Shutdown();
    ImGui::DestroyContext();
    free_app_context(lemon_app_ctx);
    glfwTerminate();

    return 0;
}