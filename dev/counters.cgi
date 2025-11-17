#!/bin/sh

source ./common.sh

if [ -z "$q" -o -z "$s" ]; then
    D="http://localhost:8000/jq.cgi?u=%7B%22counters%22%3A%5B%5D%7D&q=."
else
    D="http://localhost:8000/jq.cgi?u=$s&q=$q"
fi

# TODO: make it POSIX-only
D=$(python3 -c "from urllib.parse import quote; print(quote('$D'))")

echo "HTTP/1.1 307"
echo "Location: /hb.cgi?t=http%3A//localhost%3A8000/counters.html&d=$D"
echo
