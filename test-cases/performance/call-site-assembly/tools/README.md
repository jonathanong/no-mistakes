# Heap evidence tools

These tools inspect heaptrack 1.5.0 raw traces from the exact frozen release
ELF. They do not run inside the analyzer or change its report.

`parse-heap-raw.py` strictly replays allocation/free events and snapshots live
requested bytes at the first maximum sampled RSS record. It keeps that event
state separate from the requested-byte maximum and verifies final records,
parent trace indexes, pointer lifetimes, and aggregate byte accounting.
Symbolized frame names come from the frozen ELF's symbol table; optimized
shared generic frames do not identify object ownership on their own.

For a trace with overlapping pointer events, compile `quarantine.cpp` and pass
an uncompressed raw input, filtered output, and audit JSON path. Its first pass
audits the complete stream. Its second pass removes every allocation and free
in the full history of addresses with overlapping allocations or unknown frees.
It preserves all trace definitions and RSS records. The global recorded-payload
bound sums each excluded address's maximum logged multiplicity multiplied by
its maximum recorded payload. The strict replay must accept every remaining
address. The bound excludes never-recorded allocations and is not an RSS bound.

```sh
g++ -std=c++20 -O2 quarantine.cpp -o /tmp/nm-heap-quarantine
python3 check-audit.py /tmp/nm-heap-quarantine
zstd -d input.raw.zst -o /tmp/input.raw
/tmp/nm-heap-quarantine /tmp/input.raw /tmp/clean.raw /tmp/audit.json
zstd /tmp/clean.raw -o /tmp/clean.raw.zst
python3 parse-heap-raw.py /tmp/clean.raw.zst --binary /path/to/frozen-release
```

The saved regression traces cover clean input, unknown frees, and an overlap
after the RSS peak. Future collisions must quarantine earlier history too;
tied RSS maxima select the first record. Profiles have instrumentation overhead
and cannot replace the native `/usr/bin/time` runtime comparisons.
