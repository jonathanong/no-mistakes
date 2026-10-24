# HTML ID diagnostic ownership

`htmlIds: true` supplies HTML IDs to coverage without making them test-ID hooks.
Only `testIds: [id]` or a component test-ID mapping to `id` makes those IDs inputs
to low-effort test-ID uncertainty diagnostics. The dedicated HTML-ID rule retains
its own diagnostics. Literal test-ID duplicate findings and all max results stay
unchanged; the suppressed derived selector must remain silent in every policy.
