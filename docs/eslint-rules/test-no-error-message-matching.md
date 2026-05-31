# `no-mistakes/test-no-error-message-matching`

Disallows assertions on error message strings.

Why: exact message matching is brittle and discourages typed/domain-specific
errors.

Counterexample: `expect(error.message).toBe("invalid user")`.

Fix: assert on error type, code, shape, or a stable public field.
