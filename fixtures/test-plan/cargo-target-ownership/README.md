Cargo target ownership controls. The engine intentionally has both src/lib.rs
and src/main.rs, so src/tests.rs and src/unit/tests/mod.rs cannot choose one
suite from paths alone. Its named worker directory is specific. The flat
integration helpers directory has no entrypoint and must not invent a target.
The custom package has a renamed library, explicit custom integration entry,
disabled automatic targets, disabled tests, and a required-feature target.
Edition fixtures distinguish legacy per-target auto-discovery from modern
and inherited workspace editions, including an explicitly renamed entrypoint
whose physical filename must not create a second automatic target.
