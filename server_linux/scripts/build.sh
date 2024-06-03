SCRIPT=$(realpath -s "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
PATHTOFILES="$SCRIPTPATH/../src"
PATHTOTARGET="$SCRIPTPATH/../target"
mkdir -p ${PATHTOTARGET}

files=()
files+=("$PATHTOFILES/main.c")


g++ ${files[@]} -o ${PATHTOTARGET}/server `pkg-config --libs libuv`