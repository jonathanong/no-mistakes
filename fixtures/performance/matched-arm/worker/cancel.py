#!/usr/bin/env python3
from pathlib import Path
import sys

sys.path.insert(0, sys.argv[1])
from common import install_cancel_handler
from worker import sample

install_cancel_handler()
root = Path(__file__).parent
sample(root / 'announce.py', root, root / 'unused.yml', '1', 'off', False,
       Path(sys.argv[2]), 'b', 0)
