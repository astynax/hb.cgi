#!/bin/sh

source ./common.sh

if [ -z "$u" -o -z "$q" ]; then
    echo ""
    echo "Parameters 'u' and 'q' are required"
    exit
fi

u="$(urldecode $u)"
q="$(urldecode $q)"

echo "Content-Type: application/json"
echo ""
curl -Gs "$u" | jq -c "$q"
