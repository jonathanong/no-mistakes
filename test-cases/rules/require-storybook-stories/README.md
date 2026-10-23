# `require-storybook-stories` saved fixtures

## `fixture/malformed-selected`

This case selects malformed TSX through `options.include`. With the shared
`unanalyzable` policy set to `report`, the selected file produces one
file-level finding with target `unanalyzable` at line 1. The rule does not
infer exports from the parse-failed source or invent missing-story findings.
The same diagnostic follows the shared `advise` and `ignore` policies.

The malformed file is `Broken.tsx`. The remaining saved files protect the
selection and suppression boundaries:

- `FileDisabled.tsx` and `LineDisabled.tsx` verify file and first-line
  suppression.
- `Allowed.tsx` and `Excluded.tsx` verify the rule's allow and exclude
  controls.
- `GenericExcluded.tsx` verifies the generic rule filter.
- `_Private.tsx`, `index.tsx`, and `Broken.test.tsx` verify private, index,
  and test-file exclusions.
- `plain.ts` verifies that non-TSX/JSX files are outside this diagnostic.
- `outside/Unselected.tsx` verifies project-root selection.

The diagnostic cannot establish Storybook coverage for a parse-failed file.
Fix the syntax and use valid TSX/JSX, or deliberately remove the file from
coverage with an explicit exclude or a reasoned allow entry. A file selected
only through inferred React exports remains outside this check: its exports
cannot be established after parsing fails.
