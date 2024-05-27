#
# Cross Platform Makefile
# Compatible with MSYS2/MINGW, Ubuntu 14.04.1 and Mac OS X
#
# You will need GLFW (http://www.glfw.org):
# Linux:
#   apt-get install libglfw-dev
# Mac OS X:
#   brew install glfw
# MSYS2:
#   pacman -S --noconfirm --needed mingw-w64-x86_64-toolchain mingw-w64-x86_64-glfw
#

#CXX = g++
#CXX = clang++


EXE = target/lemon_comm
IMGUI_DIR = imgui
SOURCES_DIR = src
TARGET_DIR = target
SOURCES = $(SOURCES_DIR)/main.cpp
SOURCES += $(SOURCES_DIR)/utils.c $(SOURCES_DIR)/lemon_ctx.c $(SOURCES_DIR)/lemon_gui.cpp
SOURCES += $(IMGUI_DIR)/imgui.cpp $(IMGUI_DIR)/imgui_demo.cpp $(IMGUI_DIR)/imgui_draw.cpp $(IMGUI_DIR)/imgui_tables.cpp $(IMGUI_DIR)/imgui_widgets.cpp
SOURCES += $(IMGUI_DIR)/backends/imgui_impl_glfw.cpp $(IMGUI_DIR)/backends/imgui_impl_opengl3.cpp
OBJS = $(addprefix $(TARGET_DIR)/obj/, $(addsuffix .o, $(basename $(notdir $(SOURCES)))))
UNAME_S := $(shell uname -s)
LINUX_GL_LIBS = -lGL

CXXFLAGS = -std=c++11 -I$(IMGUI_DIR) -I$(IMGUI_DIR)/backends
CXXFLAGS += -g -Wall -Wformat
LIBS = `pkg-config --libs libuv libstrophe`

ifeq ($(UNAME_S), Linux) #LINUX
	ECHO_MESSAGE = "Linux"
	LIBS += $(LINUX_GL_LIBS) `pkg-config --static --libs glfw3`

	CXXFLAGS += `pkg-config --cflags glfw3`
	CFLAGS = $(CXXFLAGS)
endif

# ifeq ($(OS), Windows_NT)
# 	ECHO_MESSAGE = "MinGW"
# 	LIBS += -lglfw3 -lgdi32 -lopengl32 -limm32

# 	CXXFLAGS += `pkg-config --cflags glfw3`
# 	CFLAGS = $(CXXFLAGS)
# endif

##---------------------------------------------------------------------
## BUILD RULES
##---------------------------------------------------------------------

$(TARGET_DIR)/obj/%.o:$(SOURCES_DIR)/%.c
	$(CXX) $(CXXFLAGS) -c -o $@ $<

$(TARGET_DIR)/obj/%.o:$(SOURCES_DIR)/%.cpp
	$(CXX) $(CXXFLAGS) -c -o $@ $<

$(TARGET_DIR)/obj/%.o:$(IMGUI_DIR)/%.cpp
	$(CXX) $(CXXFLAGS) -c -o $@ $<

$(TARGET_DIR)/obj/%.o:$(IMGUI_DIR)/backends/%.cpp
	$(CXX) $(CXXFLAGS) -c -o $@ $<

all: $(EXE)
	@echo Build complete for $(ECHO_MESSAGE)

$(EXE): $(OBJS)
	$(CXX) -o $@ $^ $(CXXFLAGS) $(LIBS)

clean:
	rm -f $(EXE) $(OBJS)