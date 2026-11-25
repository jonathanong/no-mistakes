#!/usr/bin/env bash
for attempt in 1 2 3; do
  export ATTEMPT=$attempt
  pnpm test:unit &&
    break
done
for attempt in 1 2 3; do
  export ATTEMPT=$attempt
  ! pnpm test:unit || break
done
for attempt in 1 2 3; do
  export ATTEMPT=$attempt
  ! pnpm test:unit && sleep 1 && continue
done
for attempt in 1 2 3; do
  export ATTEMPT=$attempt
  if ! pnpm test:unit; then
    sleep 1
    continue
  fi
done
for attempt in 1 2; do
  export ATTEMPT=$attempt
  if pnpm test:unit && echo ok; then
    break
  fi
done
for shard in 1 2; do
  export SHARD=$shard
  pnpm test:unit || break
done
for shard in 1 2; do
  export SHARD=$shard
  if pnpm test:unit; then
    echo ok
  else
    break
  fi
done
for attempt in 1 2; do
  pnpm test:unit
  export ATTEMPT=$attempt
done
for pkg in a b; do
  pnpm test:unit
  pushd "$pkg"
done
for pkg in a b; do
  pushd "$pkg"
  pnpm test:unit
done
for attempt in 1 2; do
  export ATTEMPT=$attempt
  pnpm test:unit && echo passed && break
done
for attempt in 1 2; do
  export ATTEMPT=$attempt
  pnpm test:unit || # failed
    continue
done
printf '%s\n' unit |& while read -r suite; do
  pnpm test:unit "$suite"
done
while FOO=1; do
  BAR=2
  pnpm test:unit
done
while read -r suite < suites; do
  pnpm test:unit "$suite"
done
for outer in a b; do
  for inner in 1 2; do
    if true; then
      pnpm test:unit
    fi
  done
done
for shard in 1 2; do
  export SHARD=$shard
  ! pnpm test:unit
  echo done
done
for shard in 1 2; do
  export SHARD=$shard
  if pnpm test:unit || false; then
    echo ok
  fi
done
while read -r -- suite; do
  pnpm test:unit "$suite"
done
while read -a 1bad; do
  pnpm test:unit "$REPLY"
done
while read -r 1bad suite; do
  pnpm test:unit "$suite"
done
for suite in unit integration; do
  printf '%s\n' "$suite" | pnpm test:unit
done
for suite in unit integration; do
  printf '%s\n' "$suite" |&
    pnpm test:unit
done
for attempt in 1 2 3; do
  printf '%s\n' "$attempt" | pnpm test:unit && break
done
for attempt in 1 2; do
  printf '%s\n' "$attempt" | cat
  pnpm test:unit
done
for suite in unit integration; do
  pnpm test:unit | grep "$suite"
done
for attempt in 1 2; do
  export ATTEMPT=$attempt
  if
    pnpm test:unit
  then
    break
  fi
done
for attempt in 1 2; do
  export ATTEMPT=$attempt
  if ! pnpm test:unit; then
    echo failed
  else
    break
  fi
done
for attempt in 1 2; do
  export ATTEMPT=$attempt
  if pnpm test:unit; then
    echo ok
  else
    continue
  fi
done
