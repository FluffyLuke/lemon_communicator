SCRIPT=$(realpath -s "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
PATHTOFILES="$SCRIPTPATH/../src"
PATHTOUI="$SCRIPTPATH/../src/ui"
PATHTOTARGET="$SCRIPTPATH/../target"

files=()
files+=("$PATHTOFILES/main.c")

gcc `pkg-config --cflags gtk4` ${files[@]} -o ${PATHTOTARGET}/linux_client `pkg-config --libs gtk4`