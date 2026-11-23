print &>/dev/null status=argument
print &>>/dev/null status=argument
&>/dev/null status=assignment
2>/dev/null status=assignment
2 >/dev/null status=argument
10>/dev/null status=argument
'2'>/dev/null status=argument
2<(print sample) status=argument

cat <<$'END'
status=example
END
status=assignment
