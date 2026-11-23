status() { :; }
typeset -f status
typeset -h status
status=1

{
  local -h status=2
  status=3
}

outer()
{
  typeset -gh status=4
  status=5
  { local -h status=6; }
  status=7
  function inner { status=8; }
  status=9
}
status=10

function collect_scope
{
  typeset -h status=11
  typeset -i status=12
}; status=13

function { local -h status=14; status=15; }
status=16
