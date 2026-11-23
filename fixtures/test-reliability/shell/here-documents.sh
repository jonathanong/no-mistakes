#!/usr/bin/env bash
cat <<FIRST <<-"SECOND" # two documents share one command
for attempt in 1 2 3; do pnpm test:unit; done
FIRST
	for attempt in 1 2 3; do pnpm test:unit; done
	SECOND
printf '%s\n' 'for attempt in 1 2 3; do pnpm test:unit; done'
printf '%s\n' for\;attempt\;in
for attempt in 1 2; do pnpm test:unit && break; done
cat <<'UNTERMINATED'
for attempt in 1 2 3; do
  pnpm test:unit
done
