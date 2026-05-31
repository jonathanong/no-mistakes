# `no-mistakes/no-delete-property`

Disallows deleting object properties.

Why: deletion mutates object shape and makes data flow harder for agents and
tests to reason about.

Counterexample: `delete user.password`.

Fix: create a new object with the desired properties omitted.
