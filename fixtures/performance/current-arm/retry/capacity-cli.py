#!/usr/bin/env python3
import json
import sys
print(json.dumps({'ok': True}))
print('[timing] preserved raw diagnostic', file=sys.stderr)
