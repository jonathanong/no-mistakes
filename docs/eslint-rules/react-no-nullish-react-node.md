# `no-mistakes/react-no-nullish-react-node`

Disallows nullish coalescing on ReactNode-like values.

Why: `ReactNode` already includes visible falsy values, so `??` can preserve empty
strings or zeroes unexpectedly.

Counterexample: `const label: ReactNode = value ?? "Fallback"`.

Fix: use an explicit condition that matches the intended fallback behavior.
