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
    export $key="$val"
done
