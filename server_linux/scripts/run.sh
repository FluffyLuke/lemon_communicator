SCRIPT=$(realpath -s "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
PATHTOFILES="$SCRIPTPATH/../src/"
PATHTOTARGET="$SCRIPTPATH/../target"

if ${SCRIPTPATH}/build.sh ; then
    ${PATHTOTARGET}/server $@
else
    echo "Cannot run program!"
fi
