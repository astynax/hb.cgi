#!/bin/sh

if [ "$REQUEST_METHOD" = "POST" ]; then
    read INPUT
else
    INPUT="$QUERY_STRING"
fi

urldecode() {
    s=${1//+/ }
    printf '%b' "$(printf '%s' "$s" | sed 's/%/\\x/g')"
}

oldIFS=$IFS
IFS='&'
set -- $INPUT
IFS=$oldIFS

for pair; do
    key="${pair%%=*}"
    val="${pair#*=}"
    case $key in
        t) export $key="$(urldecode $val)" ;;
        d|b) export $key="$val" ;;
    esac
done

if [ -z "$d" -o -z "$b" ]; then
    echo ""
    echo "Paramaters 'd' and 'b' cannot be empty"
    exit 0
fi

if [ -n "$t" ]; then
    b=$(echo "$b" | gbase64 -w0)
    echo "Content-Type: text/html"
    echo ""
    m4 -D xT="$t" \
       -D xD="$d" \
       -D xB="$b" \
       ./get2post.html
else
    b="$(urldecode $(echo $b | b64decode -rp))"
    echo "Content-Type: application/json"
    echo ""
    echo "$b" | curl --silent -X POST --data-binary @- "$(urldecode $d)"
fi
