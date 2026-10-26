# Runner-only matcher uncertainty

Forbidden-call Playwright roots must retain matcher diagnostics even without
selector or coverage rules. Known sibling roots and explicit include overrides
remain independent of the uncertain web matcher.
