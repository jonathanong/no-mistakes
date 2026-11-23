report() {
  local status=$?
  typeset -i count status
  declare -g status=value
  status=$(collect)
  if ready; then status+=value; fi
  local exit_status=$?
  local -h status=$?
  status=value
}
status=$?
typeset -p status
typeset -m status
typeset -- status=value
[[ enabled = enabled && status=value = example ]]
print status=example
print 'status=example'
print "local status=value"
print <<< 'status=example'
print <(print status=example)
print `print status=example`
print 'status=example' >/dev/null
print 'status=example' 2>&1
(( shift = 1 << 2 ))
# status=value
pipestatus=(1 2)
cat <<-'EXAMPLE'
	status=example
	EXAMPLE
# no-mistakes-disable-next-line zsh-readonly-parameter -- Shell example fixture.
status=value
status=value # no-mistakes-disable-line zsh-readonly-parameter -- Shell example fixture.
print { status=value > \
  /dev/null
print < \
status=value
