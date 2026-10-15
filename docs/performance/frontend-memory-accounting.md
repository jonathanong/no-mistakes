<!-- cspell:ignore memtrack msgpack unpacker realloc ktime ABBA BAAB zstd venv PYTHONDONTWRITEBYTECODE Wextra Werror pthread -->

# Frontend memory capture accounting

The scripts in `.github/scripts/frontend-memory-accounting/` validate small
CodSpeed memtrack 1.6.0 captures and replay observed allocator object lifetimes.
They do not reproduce CodSpeed server metrics, native RSS, or the historical
frontend comparison in [issue #2037](https://github.com/jonathanong/no-mistakes/issues/2037).
That issue remains open: its original raw profiles and matching benchmark windows
are unavailable, and no frozen-pair repeatability experiment has been performed.

## Format and scope

The version contract is the official memtrack 1.6.0 source at
[`0576920`](https://github.com/CodSpeedHQ/codspeed/tree/0576920423ba512f75bc0d8dfc5ea09deebab165/crates/runner-shared/src/artifacts/memtrack).
Runner 5.4.0 pins this version. Its pipeline writes at most 65,536 events per
self-contained zstd frame, then concatenates frames in input order. Frame and
MessagePack object boundaries differ. Stack polling and terminal mapping records
can introduce backward timestamps; the reader preserves encoded order instead of
sorting it and inventing a global allocation order.

The upstream iterator treats a decoding error as end of iteration. This validator
instead rejects malformed/truncated records, invalid schemas, duplicate keys,
unknown extensions, and producer-count mismatches. Even a complete, valid zstd
prefix can omit a whole final frame. A count from the producer is necessary to
catch that case. A matching count establishes agreement with the producer output;
it does **not** establish absence of kernel drops or complete allocator coverage.

Limits are intentionally small: 4 MiB compressed, 16 MiB decoded, 100,000 events,
64 processes, 256 process/thread identities, 4,096 stacks totaling 8 MiB, and
32,768 live objects. Individual strings, vectors, maps, and stack payloads are
bounded before replay. The command limits Python to 256 MiB and ten CPU seconds;
its zstd child has smaller memory, output, CPU, and wall-clock limits. These are
validation limits, not performance gates. Rejection means unsupported input, not
permission to bypass a download limit or silently select a smaller favorable run.

## Run the validation

Use Linux, Python 3.12 or newer, and the zstd CLI. Install the pinned MessagePack
dependency into a temporary virtual environment:

```sh
python3 -m venv /tmp/frontend-accounting-venv
/tmp/frontend-accounting-venv/bin/pip install --only-binary=:all: --no-deps \
  -r .github/scripts/frontend-memory-accounting/requirements.txt
PYTHONDONTWRITEBYTECODE=1 /tmp/frontend-accounting-venv/bin/python \
  .github/scripts/frontend-memory-accounting/test_accounting.py
/tmp/frontend-accounting-venv/bin/python \
  .github/scripts/frontend-memory-accounting/validate_capture.py CAPTURE \
  --producer-events COUNT
```

Optional `--pid`, `--begin-index`, and `--end-index` select a half-open encoded
record window. Derive those coordinates from independently recorded markers;
never infer benchmark windows from allocation sizes in an unknown profile.
Replay requires an observed preceding process image. Object identity is process
plus address with successive generations, so cross-thread frees and address reuse
work. Successful realloc releases the known old size and allocates the new size,
including a same-address resize. Legacy missing old addresses, unmatched frees,
duplicate live addresses, null or zero-size realloc results, and process lifecycle changes make
retention, known endpoints, and peak unknown. The latest pre-window lifecycle
boundary must also be an observed Exec. Failed or zero-size realloc behavior is not validated
by the current real probe; do not assert complete history for those operations.

`--continuous-capture` and `--object-history-verified` are separate external
attestations, not facts inferred from raw records. Without both and a verified
producer count, retention remains unknown. The history attestation requires an
independent ledger/coverage argument for the selected objects, including
pre-window history and allocator interception completeness. The controlled
fixture's explicit object subset has such a ledger; the whole process does not.
Its observed stream peak is only an encoded-order sum, never server peak or RSS.
Allocated minus matched freed bytes is arithmetic over a window, not retained
object identity; a negative net can release a known pre-window object.

## Local real-capture evidence

The saved C probes were compiled with `cc -std=c11 -O0 -g
-fno-omit-frame-pointer -Wall -Wextra -Werror -pthread`. Official memtrack 1.6.0
x86 Linux release bytes were verified against the publisher checksum. Tracking
ran with a sanitized environment, a 60-second outer deadline, and no source or
binary changes between capture and replay. These are local format/accounting
probes, not ARM benchmark measurements. Raw stack payloads are not committed.
The saved JSON contains explicitly labeled derived lifetime rows, not a complete
raw capture. Tiny textual MessagePack hex cases are synthetic parser negatives.

| Probe | Raw bytes | Events | SHA-256 |
| --- | ---: | ---: | --- |
| Four-worker ledger with stacks | 68,714 | 893 | `ae8604e3a29af7a4125cbabb02340db09c4cc50c464693aa3f98aa49f109d12d` |
| 40,000 allocation/free pairs, stacks disabled | 448,060 | 80,404 | `8d187b48b20817fb09e6d5be4f3181f457d88129df770a6a72c990133bdb3efa` |

The ledger capture contains three backward timestamp transitions. Its controlled
object subset allocates 715 bytes and releases 816 bytes within the marker window:
101 bytes existed before the window, final live objects are zero, and the
encoded-order peak is 390 bytes. Four unmatched worker teardown frees make the
whole-process retained-object count and peak unknown. They are retained as
adverse evidence, not filtered out to manufacture a complete process profile.

The second capture has two zstd frames. Removing its complete final frame leaves
a valid compressed stream with 65,536 events; the expected producer count of
80,404 rejects that prefix. The version-matched real captures validate framing
and accounting mechanics only. They do not validate a custom ledger against
CodSpeed's independently defined server memory result.

## Evidence still required

For the original comparison, obtain the original small raw profiles through a
supported interface together with matching execution timestamps/benchmark
markers, tracking enable/disable history, producer/drop diagnostics, module
metadata, and the exact debug executable identity. Existing executable ZIPs exceed
the supported tool download limit and are not raw profile substitutes.

Any new repeatability measurement needs reviewed immutable source and executable
hashes, one matching ARM VM and toolchain/flags, four Rayon workers, balanced ABBA
and BAAB order, distinct original/cold and conditioned modes, all adverse and
partial observations, and a separate regex-worker hypothesis test. Neither the
historical adverse frontend observation nor #1972 causality is resolved here.
