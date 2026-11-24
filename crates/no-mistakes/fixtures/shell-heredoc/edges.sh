cat <<< "note"
value=$((1 << 2))
(( n << 1 ))
printf '%s' "$((1 << 2))" <<-'FIRST' <<"SECOND"
payload one
	FIRST
payload two
SECOND
cat <<
cat <<'MISSING'
still in the body
