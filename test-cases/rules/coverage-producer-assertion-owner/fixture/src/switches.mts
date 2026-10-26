import { strictEqual } from 'node:assert';

// The sole assertion is in a statically impossible case.
export function impossibleCase() { switch (1) { case 2: strictEqual(1, 1); break; } }
export function matchingCase() { switch (1) { case 1: strictEqual(1, 1); break; } }
export function defaultCase() { switch ('missing') { case 'other': break; default: strictEqual(1, 1); } }
export function fallthroughCase() { switch (true) { case true: ; case false: strictEqual(1, 1); } }
export function stoppedFallthrough() { switch (null) { case null: break; default: strictEqual(1, 1); } }
export function defaultBeforeMatch() { switch (1) { default: strictEqual(1, 1); break; case 1: break; } }
export function defaultFallsThrough() { switch (1) { default: ; case 2: strictEqual(1, 1); } }
export function typedCaseMismatch() { switch (1) { case '1': strictEqual(1, 1); } }
export function returnedCase() { switch (1) { case 1: return; case 2: strictEqual(1, 1); } }
export function thrownCase() { switch (1) { case 1: throw 1; case 2: strictEqual(1, 1); } }
// Unknown identities retain the existing conservative syntactic graph treatment.
export function unknownDiscriminant(value: number) { switch (value) { case 1: strictEqual(1, 1); } }
function selectedCase() { return 1; }
export function unknownCase() { switch (1) { case selectedCase(): strictEqual(1, 1); } }
export function emptySwitch() { switch (1) {} }
// Oxc encodes lone surrogates; the flag is part of literal identity.
export function loneSurrogateDefault() { switch ('\ud800') { case '\uFFFDd800': break; default: strictEqual(1, 1); } }
export function loneSurrogateMatch() { switch ('\ud800') { case '\ud800': strictEqual(1, 1); break; } }
export function loneSurrogateWrongCase() { switch ('\ud800') { case '\uFFFDd800': strictEqual(1, 1); } }

export function blockBreak() { switch (1) { case 1: { break; } case 2: strictEqual(1, 1); } }
export function blockReturn() { switch (1) { case 1: { { return; } strictEqual(1, 1); } case 2: strictEqual(1, 1); } }
export function blockFallthrough() { switch (1) { case 1: { ; } case 2: strictEqual(1, 1); } }
export function innerLoopBreak() { switch (1) { case 1: { for (;;) { break; } } case 2: strictEqual(1, 1); } }
