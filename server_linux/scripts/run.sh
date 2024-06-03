SCRIPT=$(realpath -s "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
PATHTOFILES="$SCRIPTPATH/../src/"
PATHTOTARGET="$SCRIPTPATH/../target"

${SCRIPTPATH}/build.sh
${PATHTOTARGET}/server $@