SCRIPT=$(realpath -s "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
PATHTOFILES="$SCRIPTPATH/src"
PATHTOIMGUI="$SCRIPTPATH/imgui"
PATHTOTARGET="$SCRIPTPATH/target"

mkdir -p ${PATHTOTARGET}

files=()
files+=("$PATHTOFILES/main.cpp")
#files+=("$PATHTOFILES/utils.c")
files+=("$PATHTOIMGUI/imgui.cpp")
files+=("$PATHTOIMGUI/imgui_demo.cpp")
files+=("$PATHTOIMGUI/imgui_draw.cpp")
files+=("$PATHTOIMGUI/imgui_tables.cpp")
files+=("$PATHTOIMGUI/imgui_widgets.cpp")
files+=("$PATHTOIMGUI/backends/imgui_impl_glfw.cpp")
files+=("$PATHTOIMGUI/backends/imgui_impl_opengl3.cpp")


g++ ${files[@]} -o ${PATHTOTARGET}/linux_client -g -Wall -Wformat -lGL `pkg-config --libs libuv libstrophe glfw3 imgui`