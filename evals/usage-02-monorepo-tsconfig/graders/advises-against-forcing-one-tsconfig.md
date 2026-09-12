---
type: llm
focus: last_message
weight: 1
---

The correct answer is NO for ordinary monorepo resolution.
Forcing a single package's tsconfig makes every importing file
resolve through the wrong config, which UNDER-reports consumers —
the failure mode that matters most here.

Passing: advises letting each importing file resolve through its
own owning config (i.e. not pinning one package tsconfig), and
ideally says why — pinning narrows resolution and loses
consumers.

Failing: endorses pinning `services/api/tsconfig.json` as the way
to get correct monorepo resolution, or treats it as harmless.
