---
name: dependency-rules
description: "Load before adding, upgrading, or removing a dependency, or editing package.json, a lockfile (pnpm-lock.yaml, package-lock.json), a Dependabot config, or an npm publish step. Lists the defects to avoid and the patterns that need a stated reason."
---

# Dependency and package rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Manifests and lockfiles

- MUST NOT import an undeclared dependency.
- SHOULD NOT depend on git URLs or unapproved registries.
- MUST NOT hand-edit lockfiles, or let normal builds update them; change them with the package manager on purpose.
- MUST NOT ship a package whose `exports`, `types`, `files`, and `bin` disagree with how it is installed and imported.
- SHOULD NOT add a dependency when an existing utility fits, or one with install scripts without vetting it.
- SHOULD NOT commit more than one lockfile format, or mix dependency sections.

## Publishing

- SHOULD NOT publish from a laptop or with a long-lived token once trusted publishing (OIDC) is configured; the first publish that creates the package is the exception.
- MUST NOT publish without a dry run that shows no secrets, env files, or stray files.
- MUST NOT republish an existing version or replace a module with a stub.
- MUST NOT retry auth or registry failures blindly.
- MUST NOT ship a breaking change under a non-major version of a package that promises SemVer at or above 1.0.

## Dependabot

- MUST NOT put security updates behind a cooldown.
- MUST NOT auto-merge majors, pre-1.0 minors, prereleases, or downgrades.
- MUST NOT auto-approve bot PRs; let required checks gate auto-merge.
- MUST NOT run PR code in the privileged auto-merge workflow.
