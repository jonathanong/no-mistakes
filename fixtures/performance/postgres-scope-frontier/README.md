# Inert scope frontier

`small.mts` and `large.mts` have five and 129 inert module bindings. The focused
reachability tests load these saved sources through the normal collector and
module initializer, then count actual binding scans under both root policies.
The functions deliberately remain uncalled: invoking them creates ordinary local
frames whose bindings still require traversal.

This fixture proves a production traversal reduction. It does not establish an
end-to-end speedup or complete the undo/generational evaluator design in #2113.
