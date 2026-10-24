# `require-storybook-stories` saved fixtures

## `fixture/malformed-selected`

This case selects malformed TSX through `options.include`. With the shared
`unanalyzable` policy set to `report`, the selected file produces one
file-level finding with target `unanalyzable` at line 1. The rule does not
infer exports from the parse-failed source or invent missing-story findings.
With `advise`, the diagnostic appears in advisories; `ignore` omits it.

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
- `outside/Unselected.tsx` verifies explicit include matching.

The diagnostic cannot establish Storybook coverage for a parse-failed file.
Fix the syntax and use valid TSX/JSX, or deliberately remove the file from
coverage with an explicit exclude or a reasoned allow entry. A file selected
only through inferred React exports remains outside this check: its exports
cannot be established after parsing fails.

### Malformed stories

`fixture/malformed-stories` protects application-scoped incomplete coverage:
valid stories still cover known components, malformed stories leave the other
web components unknown, and the separate project's missing component remains
a definite finding. Saved file/line exemptions exercise suppression audits;
`advise.yml` and `ignore.yml` retain the same uncertainty under each policy.
