#!/bin/sh
# Keep the hook PID across exec so the regression can prove it was reaped.
printf '%s %s\n' "$$" "$PPID" > "$NO_MISTAKES_FSMONITOR_PID"
exec sleep 60
