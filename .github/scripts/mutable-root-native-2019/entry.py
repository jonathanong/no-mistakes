"""Workflow entrypoints; DRAFT always fails before process/artifact access."""
import sys
from guard import reviewed,coverage
from common import require
reviewed()
command=sys.argv[1]
if command=='preflight':coverage()
elif command=='build':
    from build import build
    require(len(sys.argv)==4,'build arguments');build(sys.argv[2],sys.argv[3])
elif command=='semantics':
    from runtime import semantics
    semantics()
elif command=='measure':
    from sample import sample
    sample()
else:raise AssertionError('unknown command')
