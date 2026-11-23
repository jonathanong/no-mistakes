#!/usr/bin/env bash
records=()
set &>/dev/null -u
collect "${records[@]}"
set &>>/dev/null +u
collect "${records[@]}"

set -u
slots=(ready)
slots=() 2>/dev/null
collect "${slots[@]}"
slots=(ready)
slots=() 10>/dev/null
collect "${slots[@]}"
slots=(ready)
slots=() 2 >/dev/null
collect "${slots[@]}"
slots=() '2'>/dev/null
collect "${slots[@]}"
slots=() 2<(printf example)
collect "${slots[@]}"

cat <<$'END'
collect "${records[@]}"
END
collect "${records[@]}"
cat <<prefix$'\x2d\115\u004f\U00000052\x45'
collect "${records[@]}"
prefix-MORE
collect "${records[@]}"
cat <<$'SAM\'PLE'
collect "${records[@]}"
SAM'PLE
collect "${records[@]}"
cat <<END\ MARK
collect "${records[@]}"
END MARK
collect "${records[@]}"
cat <<"END\"MARK\q"
collect "${records[@]}"
END"MARK\q
collect "${records[@]}"
cat <<END\
MARK
collect "${records[@]}"
ENDMARK
collect "${records[@]}"
cat <<$'END\t\cL\x\z\u'
collect "${records[@]}"
END	{{FORM_FEED}}\x\z\u
collect "${records[@]}"
