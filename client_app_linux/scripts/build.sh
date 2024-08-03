SCRIPT=$(realpath -s "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
PATHTOFILES="$SCRIPTPATH/../src"
PATHTOTARGET="$SCRIPTPATH/../target"
PATHTOLIBS="$SCRIPTPATH/../../libs"
PATHTOOBJS="$SCRIPTPATH/../target/obj"
mkdir -p ${PATHTOTARGET}
mkdir -p ${PATHTOOBJS}

$(cd $SCRIPTPATH/../ && make)