"""Harmless physical protocol fixture, not an analyzer or a performance sample."""
from pathlib import Path
import sys
sys.stdout.write((Path(__file__).parent/'report.json').read_text())
raise SystemExit(1)
