"""Build one exact main binary for an intentional identical-binary A/A control."""
import os
from pathlib import Path
import runpy
import sys

helpers = Path(__file__).resolve().parents[1] / 'matched-arm'
sys.path.insert(0, str(helpers))
import common
import frozen
common.SOURCES = {'base': '350ada01a24c33f98696ee98de9b965dfbe6d671'}
frozen.PINS = {}
assert os.uname().machine == 'aarch64'
Path('source').symlink_to(Path('source-base').resolve(), target_is_directory=True)
try:
    sys.argv = [str(helpers / 'build.py'), 'base']
    runpy.run_path(str(helpers / 'build.py'), run_name='__main__')
finally:
    Path('source').unlink()
