# File work timing regression

`work.mts` saves 1,000 functions with object spreads, aliases, writes and exports
(176,618 bytes, 6,001 lines). The forbidden-dependency rule requests import facts
without project annotation evaluation. One file fits within the bounded top ten.

The isolated CLI test at jobs 1 and 4 compares summed file parse milliseconds to
parse user+system CPU. It allows scheduling noise (one quarter through eight
times CPU plus 20 ms) while rejecting a timer covering only the OXC parser.
The reviewed 062345e4 release reported 1.210 ms for this file against 14.419 ms
phase CPU. It also checks nested value-flow times and identical source-byte units.
The file totals are elapsed time, not per-thread CPU; this is a scope regression,
not a throughput benchmark or a promise about every workload's ratio.
