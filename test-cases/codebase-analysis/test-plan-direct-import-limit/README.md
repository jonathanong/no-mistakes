# Test plan direct-import limit

Regression for a downstream direct-import selection bug: a same-directory test that directly imports a
changed source file must stay in the `direct` group and survive a one-file
limit even when alphabetically-earlier tests are reachable only through a
two-hop import (`aaa-*.test.mts` → `src/mid.mts` → `src/dev-server.mts`).
`src/README.md` keeps a same-directory markdown link so the source is still a
documented module, but the crowding path is the two-hop import.
