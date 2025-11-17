#!/bin/sh

source ./common.sh

if [ -z "$u" -o -z "$q" ]; then
    echo "Status: 406"
    echo ""
    echo "Parameters 'u' and 'q' are required"
    exit 1
fi

u="$(urldecode $u)"
q="$(urldecode $q)"

if echo "$u" | grep -e "^http"; then
    u=$(curl -Gs "$u")
fi

echo "Content-Type: application/json"
echo ""
echo "$u" | jq -c "$q"
