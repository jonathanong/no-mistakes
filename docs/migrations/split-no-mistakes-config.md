# Split a large no-mistakes config

Use `.no-mistakes/` to move large top-level sections out of the root config.
Keep top-level scalars such as `effort` in the root file; section folders hold
the section's mapping or list contents. For example, move a root `rules:` list
to `.no-mistakes/rules/policies.yml` by removing the outer `rules:` key:

```yaml
# .no-mistakes/rules/policies.yml
- rule: no-live-repository-scan
  name: product repositories
  options:
    roots: [src]
```

The root file and fragment directory can coexist. The root file is merged first,
then fragments in sorted relative-path order. Mapping keys deep-merge; list
entries concatenate in file order, preserving each file's authored item order.
Repeated scalar keys and incompatible value types fail with both source files
identified. Repeated named entries across fragments in the same list are also
an error that identifies both files. Existing duplicate checks within one file
remain unchanged. Relative paths in every fragment are rooted at the repository
root. An explicit `--config <file>` still loads only that file.

## Update direct config readers

Code that parsed `.no-mistakes.yml` directly or matched its raw text will see
only part of the config after a split. Read the declared, merged source-level
view instead. The CLI equivalent is `no-mistakes config resolve --root .
--view declared`. The asynchronous Node API returns the declared config object:

```js
import { resolveConfig } from "no-mistakes";

const config = await resolveConfig({ root: repositoryRoot, view: "declared" });
const rule = config.rules?.find(
  (entry) => entry.rule === "no-live-repository-scan" && entry.name === "product repositories",
);
```

Load it once per policy or CI run and pass the returned object to its readers.

The declared view merges source declarations without materializing defaults.
Aliases are normalized to their canonical form. A repository with no config
returns `{}`. List order is stable: root entries first, then fragment entries
from files in sorted relative-path order, preserving item order inside each
file. Use semantic lookups such as `find` by `rule` and `name` instead of
comparing raw file text or relying on a single-file list layout.

## Update CI path filters

Any CI job that watches config changes must include all root config extensions
and the fragment tree in its path filter:

```yaml
paths:
  - .no-mistakes.yaml
  - .no-mistakes.yml
  - .no-mistakes.json
  - .no-mistakes.jsonc
  - .no-mistakes/**
```

This keeps config validation and dependent CI jobs active when only a fragment
changes.
