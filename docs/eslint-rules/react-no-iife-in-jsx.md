# `no-mistakes/react-no-iife-in-jsx`

Disallows immediately invoked functions inside JSX.

Why: JSX IIFEs obscure render-time data flow and complicate static component
analysis.

Counterexample: `{(() => computeLabel())()}` inside JSX.

Fix: move the expression to a named variable or component helper before the JSX
return.
