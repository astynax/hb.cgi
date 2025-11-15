#!/bin/sh

source ./common.sh

if [ -z "$d" -o -z "$b" ]; then
    echo ""
    echo "Paramaters 'd' and 'b' cannot be empty"
    exit 0
fi

t=$(urldecode "$t")

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
