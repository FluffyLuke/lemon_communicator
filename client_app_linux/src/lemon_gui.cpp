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
#include "includes/lemon_ctx.h"
#include "includes/lemon_gui.h"

#include "../../libs/imgui/imgui.h"
#include "../../libs/imgui/backends/imgui_impl_glfw.h"
#include "../../libs/imgui/backends/imgui_impl_opengl3.h"
#include <GLFW/glfw3.h>

#define DEMO_POPUP_ID "demo_popup"

void render_friends_window(lemon_app_ctx* ctx, ImVec2 size, ImVec2 position) {
    lemon_gui_ctx* g_ctx = ctx->gui_ctx;
    lemon_client_ctx* c_ctx = ctx->client_ctx;

    ImGuiWindowFlags flags = 
        ImGuiWindowFlags_NoMove | 
        ImGuiWindowFlags_NoResize | 
        ImGuiWindowFlags_NoCollapse;

    ImGui::SetNextWindowPos(position);
    ImGui::SetNextWindowSize(size);

    ImGui::Begin("Friends list", NULL, flags);
    ImGui::Text("friends");
    ImGui::End();

}

void render_chat_window(lemon_app_ctx* ctx, ImVec2 size, ImVec2 position) {
    lemon_gui_ctx* g_ctx = ctx->gui_ctx;
    lemon_client_ctx* c_ctx = ctx->client_ctx;
    
    ImGuiWindowFlags flags = 
        ImGuiWindowFlags_NoMove | 
        ImGuiWindowFlags_NoResize | 
        ImGuiWindowFlags_NoCollapse;

    ImGui::SetNextWindowPos(position);
    ImGui::SetNextWindowSize(size);

    ImGui::Begin("Chat window", NULL, flags);
    ImGui::Text("chat");
    ImGui::End();
}

void render_options_window(lemon_app_ctx* ctx, ImVec2 size, ImVec2 position) {
    lemon_gui_ctx* g_ctx = ctx->gui_ctx;
    lemon_client_ctx* c_ctx = ctx->client_ctx;

    ImGuiWindowFlags flags = 
        ImGuiWindowFlags_NoMove | 
        ImGuiWindowFlags_NoResize | 
        ImGuiWindowFlags_NoCollapse;

    ImGui::SetNextWindowPos(position);
    ImGui::SetNextWindowSize(size);

    ImGui::Begin("Options", NULL, flags);

    if(g_ctx->ifDemoWindow) {
        if(ImGui::Button("Hide demo window"))
            g_ctx->ifDemoWindow = false;
    } else {
        if(ImGui::Button("Show demo window"))
            g_ctx->ifDemoWindow = true;
    }
    ImGui::SameLine();
    if(ImGui::Button("Display demo popup"))
            ImGui::OpenPopup(DEMO_POPUP_ID);
    if(ImGui::BeginPopupModal(DEMO_POPUP_ID)) {
        ImGui::Text("This is demo popup!");    
        if (ImGui::Button("Close"))
            ImGui::CloseCurrentPopup();
        ImGui::EndPopup();
    }
    ImGui::End();
}

void render_popup(lemon_app_ctx* ctx, ImVec2 size, ImVec2 position) {
    
}

void ui_main(uv_idle_t* handle) {
    lemon_app_ctx* ctx = (lemon_app_ctx*)handle->data;
    lemon_gui_ctx* g_ctx = ctx->gui_ctx;
    lemon_client_ctx* c_ctx = ctx->client_ctx;
    ImGuiIO io = ImGui::GetIO();
    ImGuiViewport* vp = ImGui::GetMainViewport();
    
    if(glfwWindowShouldClose(g_ctx->window)) {
        uv_idle_stop(handle);
    }

    glfwPollEvents();

    // Start the Dear ImGui frame
    ImGui_ImplOpenGL3_NewFrame();
    ImGui_ImplGlfw_NewFrame();
    ImGui::NewFrame();

    ImVec2 position;
    ImVec2 size;
    // --- Render GUI ---
    // Friends window
    float friends_window_pos_x = vp->WorkPos.x;
    float friends_window_pos_y = vp->WorkPos.y;
    float friends_window_size_x = 300;
    float friends_window_size_y = vp->WorkSize.y-70;
    position = ImVec2(friends_window_pos_x, friends_window_pos_y);
    size = ImVec2(friends_window_size_x, friends_window_size_y);
    render_friends_window(ctx, size, position);

    // Chat window
    float chat_window_pos_x = vp->WorkPos.x + friends_window_size_x;
    float chat_window_pos_y = vp->WorkPos.y;
    float chat_window_size_x = vp->WorkSize.x - friends_window_size_x;
    float chat_window_size_y = vp->WorkSize.y-70;
    position = ImVec2(chat_window_pos_x, chat_window_pos_y);
    size = ImVec2(chat_window_size_x, chat_window_size_y);
    render_chat_window(ctx, size, position);

    // Options window
    float options_window_pos_x = vp->WorkPos.x;
    float options_window_pos_y = vp->WorkPos.y + friends_window_size_y;
    float options_window_size_x = vp->WorkSize.x;
    float options_window_size_y = 70;
    position = ImVec2(options_window_pos_x, options_window_pos_y);
    size = ImVec2(options_window_size_x, options_window_size_y);
    render_options_window(ctx, size, position);

    // Demo window for guidance
    if(g_ctx->ifDemoWindow) {
        ImGui::ShowDemoWindow();
    }
    
    ImGui::Render();
    int display_w, display_h;
    glfwGetFramebufferSize(g_ctx->window, &display_w, &display_h);
    glViewport(0, 0, display_w, display_h);
    glClear(GL_COLOR_BUFFER_BIT);
    ImGui_ImplOpenGL3_RenderDrawData(ImGui::GetDrawData());

    glfwSwapBuffers(g_ctx->window);
}