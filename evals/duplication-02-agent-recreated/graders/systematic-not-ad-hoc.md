---
type: llm
focus: last_message
weight: 1
---

Passing: the plan proposes a systematic repository-wide check for
duplicate exported names, and recognises that per-file linting
cannot see cross-file uniqueness so this needs a whole-repo pass.

Failing: the plan offers only ad-hoc greps for specific names the
user must think of first.
