#!/bin/sh

set -xe

gcc -c -o lemon_ctx.o ./src/lemon_ctx.c
gcc -c -o lemon_client.o ./src/lemon_client.c
gcc -c -o utils.o ./src/utils.c
gcc -c -o server_tcp.o ./src/server_tcp.c
gcc -c -o vec.o ../libs/vec/src/vec.c
gcc -c -o parser.o ../libs/api/src/parser.cpp

g++ -c -o main.o ./src/main.cpp
g++ -c -o lemon_gui.o -I../libs/imgui/ ./src/lemon_gui.cpp
g++ -c -o imgui.o -I../libs/imgui/ ../libs/imgui/imgui.cpp 
g++ -c -o imgui_draw.o -I../libs/imgui/ ../libs/imgui/imgui_draw.cpp
g++ -c -o imgui_widgets.o -I../libs/imgui/ ../libs/imgui/imgui_widgets.cpp
g++ -c -o imgui_tables.o -I../libs/imgui/ ../libs/imgui/imgui_tables.cpp
g++ -c -o imgui_demo.o -I../libs/imgui/ ../libs/imgui/imgui_demo.cpp
g++ -c -o imgui_impl_glfw.o -I../libs/imgui/ ../libs/imgui/backends/imgui_impl_glfw.cpp
g++ -c -o imgui_impl_opengl3.o -I../libs/imgui/ ../libs/imgui/backends/imgui_impl_opengl3.cpp
g++ -c -o pugixml.o ../libs/pugixml/src/pugixml.cpp

g++ -o dupa lemon_ctx.o lemon_client.o utils.o main.o lemon_gui.o imgui.o imgui_draw.o imgui_widgets.o imgui_tables.o imgui_demo.o imgui_impl_glfw.o imgui_impl_opengl3.o vec.o server_tcp.o parser.o pugixml.o -luv -lGL -lglfw -lc -lm -ldl -lpthread


