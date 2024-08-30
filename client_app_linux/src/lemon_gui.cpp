#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <netinet/in.h>
#include <stdint.h>
#include <stdlib.h>
#include <sys/types.h>
#include <unistd.h>
#include <locale.h>
#include <string.h>
#include <stdio.h>
#include <chrono>

#include <uv.h>
#include "../includes/utils.h"
extern "C" {
#include "../includes/lemon_ctx.h"
}
#include "../includes/lemon_gui.h"
#include "../includes/server_tcp.h"

#include "../../libs/imgui/imgui.h"
#include "../../libs/imgui/backends/imgui_impl_glfw.h"
#include "../../libs/imgui/backends/imgui_impl_opengl3.h"
#include <GLFW/glfw3.h>
#include <uv/unix.h>

#define DEMO_POPUP_ID "demo_popup"
#define CONNECT_POPUP_ID "connect_to_server"

void render_friends_window(lemon_ctx* ctx, ImVec2 size, ImVec2 position) {
    lemon_gui_ctx* g_ctx = &ctx->gui;
    lemon_client_ctx* c_ctx = &ctx->client_ctx;

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

void render_chat_window(lemon_ctx* ctx, ImVec2 size, ImVec2 position) {
    lemon_gui_ctx* g_ctx = &ctx->gui;
    lemon_client_ctx* c_ctx = &ctx->client_ctx;
    
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

void render_demo_popup() {
    ImGui::Text("This is demo popup!");    
    if (ImGui::Button("Close"))
        ImGui::CloseCurrentPopup();
    ImGui::EndPopup();
}


void render_options_window(lemon_ctx* ctx, ImVec2 size, ImVec2 position) {    
    lemon_gui_ctx* g_ctx = &ctx->gui;
    lemon_client_ctx* c_ctx = &ctx->client_ctx;

    ImGuiWindowFlags flags = 
        ImGuiWindowFlags_NoMove | 
        ImGuiWindowFlags_NoResize | 
        ImGuiWindowFlags_NoCollapse;

    ImGui::SetNextWindowPos(position);
    ImGui::SetNextWindowSize(size);

    ImGui::Begin("Options", NULL, flags);


    if(g_ctx->if_demo_window) {
        if(ImGui::Button("Hide demo window"))
            g_ctx->if_demo_window = false;
    } else {
        if(ImGui::Button("Show demo window"))
            g_ctx->if_demo_window = true;
    }

    ImGui::SameLine();

    if(ImGui::Button("Display demo popup"))
        ImGui::OpenPopup(DEMO_POPUP_ID);
    if(ImGui::BeginPopupModal(DEMO_POPUP_ID))
        render_demo_popup();

    ImGui::SameLine();

    if(ImGui::Button("Connect to server")) {
        ImGui::OpenPopup(CONNECT_POPUP_ID);
    }
    if(ImGui::BeginPopupModal(CONNECT_POPUP_ID)) {

        //uv_mutex_lock(&conn_req->lock);
        connection_state current_state = check_connection_state(ctx);
        static tcp_request_t* conn_req = NULL;
        if(current_state == NO_CONNECTION) {
            static char addr_buff[15] = {0};
            ImGui::InputText("Provide address", addr_buff, 15);
            static int32_t port_buf = 0;
            if(port_buf > 65535) port_buf = 65535;
            if(port_buf < 0) port_buf = 0;
            ImGui::InputInt("Provide port", &port_buf, 1, 4);

            static int32_t result;
            static int8_t clicked = 0;
            if(ImGui::Button("Connect")){
                uv_rwlock_wrlock(&ctx->tcp.connection_state_lock);
                result = uv_ip4_addr(addr_buff, port_buf, &ctx->tcp.dest);
                uv_rwlock_wrunlock(&ctx->tcp.connection_state_lock);
                clicked = 1;
            }
            if(result == 0 && clicked) {
                clicked = 0;
                conn_req = (tcp_request_t*)malloc(sizeof(tcp_request_t));
                init_tcp_request(conn_req);
                conn_req->action = CONNECT_REQUEST;
                push_request(ctx, conn_req);
            } else if (result != 0 && clicked) {
                ImGui::Text("Wrong address/port!");
            }
            //uv_mutex_unlock(&conn_req->lock);
        } else if(ctx->tcp.conn_state == CONNECTING) {
            ImGui::Text("Connecting...");
        } else if(ctx->tcp.conn_state == CONNECTED) {
            ImGui::Text("Connected!");
        } else if(ctx->tcp.conn_state == CANNOT_CONNECT) {
            ImGui::Text("Couldn't connect!");
            if(ImGui::Button("Try again")) {
                tcp_request_t* conn_req = (tcp_request_t*)malloc(sizeof(tcp_request_t));
                init_tcp_request(conn_req);
                conn_req->action = DISCONNECT_REQUEST;
                push_request(ctx, conn_req);
            }
        }

        if(ImGui::Button("Close")) {
            ImGui::CloseCurrentPopup();
            if(conn_req != NULL) {
                destroy_tcp_request(conn_req);
                free(conn_req);
            }
        }

        //uv_rwlock_rdunlock(&ctx->tcp.connection_state_lock);

        ImGui::EndPopup();
    }

    connection_state state = check_connection_state(ctx);
    switch (state) {
        case CONNECTED: {
            ImGui::TextColored(ImVec4(0.0f, 1.0f, 0.4f, 1.0f), "Connected");
            break;
        }
        case CONNECTING: {
            ImGui::TextColored(ImVec4(1.0f, 0.8f, 0.0f, 1.0f), "Connecting...");
            break;
        }
        default: {
            ImGui::TextColored(ImVec4(1.0f, 0.1f, 0.0f, 1.0f), "Disconnected");
            break;
        }
    }

    ImGui::End();
}

void ui_main(uv_idle_t* handle) {
    lemon_ctx* ctx = (lemon_ctx*)handle->data;
    lemon_gui_ctx* g_ctx = &ctx->gui;
    lemon_client_ctx* c_ctx = &ctx->client_ctx;
    ImGuiIO io = ImGui::GetIO();
    ImGuiViewport* vp = ImGui::GetMainViewport();
    
    if(glfwWindowShouldClose(g_ctx->window)) {
        uv_stop(ctx->loop);
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
    if(g_ctx->if_demo_window) {
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
