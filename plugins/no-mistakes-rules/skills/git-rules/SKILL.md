---
name: git-rules
description: "MUST NOT / SHOULD NOT rules for git, commits, pull requests, docs, and change hygiene: staging, history rewrites, rebases, hooks, scope, and PR descriptions. Use when committing, rebasing, pushing, writing a PR, or updating docs and agent instructions."
---

# Git and change rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Git

- MUST NOT commit credentials, build output, editor state, or unrelated changes.
- MUST NOT bypass hooks (`--no-verify`, disabled hooks path) or plain force-push; use `--force-with-lease` against the tip you saw.
- MUST NOT amend or rewrite shared history or another author's commits without permission.
- MUST NOT rebase onto or diff against a stale local `main`; use the fetched remote branch.
- MUST NOT rebase over a dirty worktree.
- MUST NOT resolve conflicts by taking a whole side; keep both invariants and rerun their tests.
- MUST NOT change git identity or config in automation.

## Changes and PRs

- MUST NOT remove a timeout, deadline, or guard without a replacement on the same path.
- MUST NOT leave docs, fixtures, types, or generated files stale when behavior changes.
- MUST NOT close issues a PR does not fully resolve.
- MUST NOT add broad, unexplained suppressions or allowlists to get a check green.
- SHOULD NOT widen scope for a discovered follow-up; file it.
- SHOULD NOT split related work across PRs without a risk reason.
- SHOULD NOT claim performance or cost wins without measurements.
- SHOULD NOT list routine passing checks in PR descriptions; state the problem, the change, and the impact.

## Docs and agent instructions

- SHOULD NOT restate code, config values, or numbers that drift; document why and unenforced invariants.
- SHOULD NOT put history, plans, or private details in public docs.
- SHOULD NOT let agent instruction files grow past their size budget, or duplicate a shared instruction file.
