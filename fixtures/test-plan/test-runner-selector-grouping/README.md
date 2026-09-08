# Test runner selector grouping

The Cargo fixture has two integration targets with different `--test` values,
and the Swift fixture has two test targets with different `--filter` values.
Both are selected together so execution-target grouping must retain each
runner selector.
