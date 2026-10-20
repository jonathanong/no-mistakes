"""Fresh exact-main merge-base control, sharing the audited release build contract."""
from pathlib import Path
import runpy
import sys

import inputs
import common
import frozen

MAIN = '22a294398a082d08a4167cb79d9499d37b70f7bb'
common.SOURCES = {'control': MAIN}
frozen.PINS = {}
sys.argv = ['control_build_impl.py', 'control']
runpy.run_path(str(Path(__file__).with_name('control_build_impl.py')), run_name='__main__')
