import { db } from '@vendor/replacement-loop-fallbacks';
let fromNonliteral, fromHole, scalar, accepted;
// Opaque values and array holes cannot prove a callable assignment executes.
for (const item of [unresolved, 2]) { fromNonliteral = () => item; }
for (const item of [, ,]) { fromHole = () => item; }
// A scalar RHS and a member target must retain conservative fallback behavior.
for (const item of [1, 2]) { scalar = 0; }
for (const item of [1, 2]) { db.query = () => item; }
// This literal, straight-line callable assignment is the positive control.
for (const item of [1, 2]) { accepted = () => item; }
