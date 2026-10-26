# `pnpm-overrides-ban`

Ban pnpm dependency version overrides. `packageExtensions` remains allowed
because it corrects dependency metadata without forcing versions.

```yaml
rules:
  - rule: pnpm-overrides-ban
    scope: repository
```

Counterexample: `pnpm-workspace.yaml` with a top-level `overrides` map, or a
workspace `package.json` with top-level `overrides` or `pnpm.overrides`.

Fix: remove the override and fix dependency metadata upstream, or upgrade the
parent dependency instead. Keep `packageExtensions` when a package needs
corrected peer metadata.

## Why and when

Use this rule in workspaces where reproducible dependency resolution matters
more than forcing a transitive version from the root manifest.

## What it catches/requires

It rejects pnpm `overrides` in `pnpm-workspace.yaml` and workspace manifests,
including `pnpm.overrides`. A local `include` of `./` or `../` YAML that owns
`overrides` is read from the workspace root, and an `overrides` map that
exists only in that file is banned the same way. When the root file and a
local include both claim `overrides`, every file that defines the key is
reported and the check also reports that the setting is claimed twice.
`packageExtensions` remains allowed because it describes missing metadata
rather than selecting a version.

## Options and defaults

There are no rule-local options. The rule scans the configured repository
workspace manifests.

## Valid example

```yaml
# pnpm-workspace.yaml
packageExtensions:
  "react-dom@*":
    peerDependencies:
      react: "*"
```

## Counterexample

```yaml
# pnpm-workspace.yaml
overrides:
  lodash: 4.17.21
```

## Fix

Resolve the version conflict in the owning package, upgrade the dependency, or
correct its metadata with `packageExtensions`.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

A single leading UTF-8 byte order mark in `package.json` is accepted.

Selected package manifests, workspace YAML, and declared included override files that cannot be read or parsed produce an `unanalyzable` diagnostic containing the file and error. Repair malformed JSON/YAML or restore read access so dependency policy can be checked. Absent optional inputs are allowed unless explicitly configured.

`options.unanalyzable` uses the shared analysis policy: `report` (default) emits a finding, `advise` emits an advisory, and `ignore` omits the uncertain result. Definite violations remain findings under every policy. Read/parse failures are file-level diagnostics at line 1; use a file suppression for these failures. File suppressions apply before policy routing; malformed JSON can carry a leading `// no-mistakes-disable-file pnpm-overrides-ban` directive, and YAML supports the corresponding `#` directive. Rule include/exclude and project scope still limit the selected files. A custom rule `message` prefixes these diagnostics. When only included files are selected, the root workspace supplies declarations as context; diagnostics stay on selected declared files.

## Suppression

Use `no-mistakes-disable-file pnpm-overrides-ban` only when an external release
constraint makes the override unavoidable; document that constraint nearby.

## Related rules

[`version-pin-consistency`](version-pin-consistency.md) keeps intentional pins
in sync, while [`production-dependency-declarations`](production-dependency-declarations.md)
checks that runtime imports are declared in the right field.

## Limitations

Settings owned by a pnpm config dependency cannot be loaded without an install.
This rule does not read `node_modules`. A source whose key does not start with
`./` or `../` is ignored, so an `overrides` map that exists only there is not
a finding.

```yaml
include:
  "@acme/pnpm-policy":
    - overrides
```

`packageConfigs.<name>.overrides` is applied by pnpm only when
`sharedWorkspaceLockfile` is false. This rule does not read `packageConfigs`.

```yaml
sharedWorkspaceLockfile: false
packageConfigs:
  "@acme/app":
    overrides:
      lodash: 4.17.21
```
