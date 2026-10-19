# Swift missing dependency baseline

Synthetic package root. `tests.swift.packages` lists `.` because Swift
dependency analysis only walks declared package roots. An empty `packages`
list does not read `Package.swift` or `Package.resolved`.

There is no comparison baseline. The two paths differ, and they are the same
missing baseline, so the plan reports one `dependency-baseline-missing` entry.
Full-suite selection stays off.
