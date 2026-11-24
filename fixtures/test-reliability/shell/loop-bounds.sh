#!/usr/bin/env bash
for attempt in {4..1}; do
  if CI=1 pnpm test:unit -- --changed; then
    break
  fi
done
for attempt; do pnpm test:unit || exit 1; done
for attempt in ${ATTEMPTS}; do pnpm test:unit && break; done
while [ -n "$try_again" ]; do ! pnpm test:unit && break; done
for suite in unit integration; do pnpm test:unit --project "${suite}"; done
for attempt in 1 2; do pnpm test:unit --reporter "$attempt_format" && break; done
for attempt in {a..c}; do pnpm test:unit && break; done
for attempt in {1..5..2}; do pnpm test:unit && break; done
while read -r suite; do pnpm test:unit --project "$suite"; done
while IFS=, read -r project shard < suites; do SHARD=$shard pnpm test:unit; done
while read -r -d '' -- suite; do pnpm test:unit --project "$suite"; done
while read -a suites; do pnpm test:unit --project "${suites[0]}"; done
while read <suites; do pnpm test:unit --project "$REPLY"; done
while read -r suite; do pnpm test:unit && break; done
for suite in unit integration; do SUITE=$suite pnpm test:unit; done
for attempt in 1 2; do SUITE=unit pnpm test:unit && break; done
while ! pnpm test:unit; do echo retry; done
