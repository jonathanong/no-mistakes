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
[[ enabled = enabled && status=value = example ]]
print status=example
print 'status=example'
print "local status=value"
# status=value
pipestatus=(1 2)
cat <<-'EXAMPLE'
	status=example
	EXAMPLE
# no-mistakes-disable-next-line zsh-readonly-parameter -- Shell example fixture.
status=value
status=value # no-mistakes-disable-line zsh-readonly-parameter -- Shell example fixture.
