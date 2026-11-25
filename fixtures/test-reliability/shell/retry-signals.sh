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
