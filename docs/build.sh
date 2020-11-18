#!/bin/bash

DST=_html

while getopts ":hc" opt; do
  case ${opt} in
    h )
      cat <<EOF
Usage:
  $0 [-c]
Options:
  -c: remove the ${DST} folder before building
EOF
      exit 0
      ;;
    c )
      echo -n "cleaning docs "
      rm -rf ${DST}
      echo "[ok]"
      ;;
    \? )
      echo "Invalid Option: -$OPTARG" 1>&2
      exit 1
      ;;
  esac
done
shift $((OPTIND -1))

echo -n "building docs "
for MD_FILE in *.md; do
    rustdoc -o ${DST} ${MD_FILE}
done
echo "[ok]"
