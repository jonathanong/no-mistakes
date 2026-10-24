# Next.js raw source scope

This small saved fixture models two selected Next.js applications, one whose
root is inferred from `next.config.mjs`, plus an unselected sibling app. The
route rule needs raw source for route candidates only. Its suppressed,
malformed, and excluded route files remain candidates. Raw-source-only checking
reports the malformed route because it does not parse it; the mixed check's
fact parser rejects that malformed source. The invalid UTF-8 and dense ordinary
modules are outside route demand, and the sibling route should not be read.

`mixed.yml` adds the caching rule as a source-fact consumer. That request must
retain the ordinary source and parse work required by that rule while still
sharing route findings with the raw-source consumer.
