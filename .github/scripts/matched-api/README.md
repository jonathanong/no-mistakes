# Memory-only replay of frozen asynchronous API artifacts

Reuses official run39039054266 attempt1 sourcef5d01415971a302332e25741326566035e0b1a65 addons, without Rust builds. Each fixed official ZIP digest and full unpacked inventory is independently authenticated after the pinned cross-run download action. Original producer receipts retain their original run identity; new rows retain the replay run identity.

Same24 cells and192 rows, fixtures, cases, diagnostics, worker settings, fresh-process ABBA/BAAB orders and full parity controls as the original experiment. No outcome-driven retuning. Memory-only summaries use Linux exec-mm `/proc/self/status` VmHWM at immediate API completion. The full raw status is retained. Inherited `resourceUsage().maxRSS` remains a diagnostic only. VmHWM is process memory, not cache-exclusive allocation accounting.

Replay CPU/wall fields are retained as descriptive operational evidence only. Original run CPU/wall remain the sole current performance evidence, with no pooling or replacement. Historical RSS values remain visible as inherited-floor limited. No automatic performance acceptance verdict is introduced.
