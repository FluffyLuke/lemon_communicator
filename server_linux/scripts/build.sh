SCRIPT=$(realpath -s "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
PATHTOFILES="$SCRIPTPATH/../src"
PATHTOTARGET="$SCRIPTPATH/../target"
PATHTOLIBS="$SCRIPTPATH/../../libs"
mkdir -p ${PATHTOTARGET}

files=()
files+=("$PATHTOFILES/main.cpp")
files+=("$PATHTOFILES/database.c")
files+=("$PATHTOFILES/client.c")
files+=("$PATHTOFILES/requests.cpp")
files+=("$PATHTOLIBS/api/parser.cpp")

g++ ${files[@]} -o ${PATHTOTARGET}/server `pkg-config --libs libuv pugixml` $(mariadb_config --include --libs)