SCRIPT=$(realpath -s "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
PATHTOFILES="$SCRIPTPATH/../src"
PATHTOTARGET="$SCRIPTPATH/../target"
PATHTOLIBS="$SCRIPTPATH/../../libs"
mkdir -p ${PATHTOTARGET}

files=()
files+=("$PATHTOFILES/requests.cpp")
files+=("$PATHTOFILES/main.cpp")
files+=("$PATHTOFILES/database.c")
files+=("$PATHTOFILES/client.c")

files+=("$PATHTOLIBS/api/parser.cpp")
files+=("$PATHTOLIBS/vec/src/vec.c")

g++ ${files[@]} -o ${PATHTOTARGET}/server `pkg-config --libs libuv pugixml openssl` $(mariadb_config --include --libs)