---
name: git-rules
description: "MUST NOT / SHOULD NOT rules for git, commits, pull requests, docs, evidence for claims, and change hygiene: staging, history rewrites, rebases, hooks, scope, and PR descriptions. Use when committing, rebasing, pushing, writing a PR, updating docs and agent instructions, or making a claim about production behavior."
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
- MUST NOT let a new branch track the default branch as its upstream; a bare `git push` then targets it.
- MUST NOT `git stash` in a worktree someone else is editing; commit a WIP or use your own worktree.

## Changes and PRs

- MUST NOT remove a timeout, deadline, or guard without a replacement on the same path.
- MUST NOT leave docs, fixtures, types, or generated files stale when behavior changes.
- MUST NOT close issues a PR does not fully resolve.
- MUST NOT add broad, unexplained suppressions or allowlists to get a check green.
- MUST NOT ship a breaking API change without calling it breaking; removed fields, changed meanings, and blocked workflows that callers depend on all count.
- MUST NOT call a change "unchanged" from the diff alone; check the behavior every caller sees.
- SHOULD NOT dismiss a review finding with less evidence than it took to raise it.
- SHOULD NOT widen scope for a discovered follow-up; file it.
- SHOULD NOT take on scope a ticket's technical design added beyond the reported problem without asking whether the fix needs it.
- SHOULD NOT split related work across PRs without a risk reason.
- SHOULD NOT claim performance or cost wins without measurements.
- SHOULD NOT list routine passing checks in PR descriptions; state the problem, the change, and the impact.
- SHOULD NOT write a PR description without the blast radius and the paths each flag covers.

## Evidence

- MUST NOT treat an empty result from an unproven query as absence; first show the query can return rows.
- MUST NOT state a rate without its denominator.
- MUST NOT treat a capped page of results as the whole population.
- MUST NOT repeat a number from a subagent's or tool's prose summary; read the raw output.
- MUST NOT call a fix verified because a lower environment shows no error.
- MUST NOT treat "rare in production" as "unreachable"; they need different proof.

## Docs and agent instructions

- SHOULD NOT restate code, config values, or numbers that drift; document why and unenforced invariants.
- SHOULD NOT put history, plans, or private details in public docs.
- SHOULD NOT let agent instruction files grow past their size budget, or duplicate a shared instruction file.
