This saved Unix pnpm-layout counterexample intentionally uses package symlinks.
The real package siblings declare Error ancestry; the hoisted dep is benign.
Both bare and parent-relative imports must remain uncertain unless realpath
resolution semantics are owned. They must not select the hoisted benign base.
