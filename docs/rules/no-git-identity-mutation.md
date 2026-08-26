# `no-git-identity-mutation`

## Why and when

Use this rule when repository automation must not silently replace the user's
Git identity, which can misattribute commits and leak bot configuration across
jobs.

## What it catches

It catches `git config` commands that write `user.name` or `user.email` in
selected scripts; read-only identity queries remain allowed.

## Options

`include`, `exclude`, `allow`, and `message` select scripts, known exceptions,
and an optional finding hint. The documented script extension set is the
default include scope.

## Valid example

`git config user.email` without a value reads identity and passes; configuring
identity outside the repository script also passes.

## Suppression

Use a line directive for an exceptional command or a file directive only for a
dedicated setup script; prefer moving the mutation out.

## Related rules

[`shellcheck-runner`](shellcheck-runner.md) checks shell correctness but not
repository identity policy.

Bans scripts that mutate git user identity.

```yaml
rules:
  - rule: no-git-identity-mutation
    scope: repository
```

Counterexample: `git config user.email bot@example.com` in setup scripts.

Fix: read git identity when needed, but configure identity outside repository
scripts.
