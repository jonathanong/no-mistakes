# Finite set effort matrix

`default.yml` and `max.yml` are controls with the same four findings: the YAML
sequence is missing `beta`, the TypeScript `Other` union has two set differences,
and the dynamic scheduler ID is an extraction finding. The YAML selector and
compatible TypeScript union pass.

At `low-report`, the supported YAML mismatch remains one finding, the selector
comparison passes, and each of the compatible union, mismatching union, and
dynamic scheduler sets produces one unanalyzable finding. `low-advise` keeps
the definite mismatch and moves those three residuals to advisories;
`low-ignore` keeps only the definite mismatch. Unsupported sets are never
compared, so the different TypeScript members cannot create guessed mismatches.

`mixed.yml` applies low effort to the YAML mismatch and a compatible-but-
unsupported union, and max effort to the TypeScript union mismatch.
`custom-message.yml` confirms a rule-level message prefix does not change
finding/advisory classification. `low-suppression.yml` puts an intentionally
different unsupported array under a file suppression. Only the separate YAML
mismatch remains; suppressing its residual cannot authorize a comparison.
`low-min-size-ignore.yml` selects a TypeScript call extractor with no matching
calls and `minSize: 1`. Low ignore does not apply `minSize` to an unsupported
empty extraction.
