# Nested-compatible uncertainty control

The selected producer and consumer schemas are identical, so `default.yml` and
`max.yml` have no findings. Low cannot prove their nested array shape:
`low-report.yml` has one finding, `low-advise.yml` has zero findings and one
advisory, and `low-ignore.yml` has no output. Ignoring this uncertainty does not
establish that the schemas are compatible.
