SCRIPT=$(realpath -s "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
PATHTOFILES="$SCRIPTPATH/../src"
PATHTOTARGET="$SCRIPTPATH/../target"
PATHTOLIBS="$SCRIPTPATH/../../libs"
mkdir -p ${PATHTOTARGET}

files=()
files+=("$PATHTOFILES/requests.cpp")
files+=("$PATHTOFILES/main.cpp")

files+=("$PATHTOLIBS/api/src/database.c")
files+=("$PATHTOLIBS/api/src/client.c")
files+=("$PATHTOLIBS/api/src/parser.cpp")
files+=("$PATHTOLIBS/vec/src/vec.c")

g++ ${files[@]} -o ${PATHTOTARGET}/server `pkg-config --libs libuv pugixml openssl` $(mariadb_config --include --libs)