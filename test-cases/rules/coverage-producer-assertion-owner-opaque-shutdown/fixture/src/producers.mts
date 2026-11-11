import { strictEqual } from 'node:assert';
import { thenShutdown, catchShutdown, finallyShutdown, opaqueThenShutdown, opaqueCatchShutdown, opaqueFinallyShutdown } from './registry.mts';

import { thenShutdown as unresolvedThenShutdown, catchShutdown as unresolvedCatchShutdown, finallyShutdown as unresolvedFinallyShutdown } from './registry-opaque-callback.mts';

function assertResult() { strictEqual(1, 1); }
async function main() { assertResult(); }

export function ownedThen() { assertResult(); thenShutdown(); }
export function ownedCatch() { assertResult(); catchShutdown(); }
export function ownedFinally() { return main().finally(finallyShutdown); }
export function unownedThen() { return opaqueThenShutdown(); }
export function unownedCatch() { return opaqueCatchShutdown(); }
export function unownedFinally() { return opaqueFinallyShutdown(); }

export function ownedOpaqueThen() { assertResult(); opaqueThenShutdown(); }
export function ownedOpaqueCatch() { assertResult(); opaqueCatchShutdown(); }
export function ownedOpaqueFinally() { return main().finally(opaqueFinallyShutdown); }

export function ownedUnresolvedThen() { assertResult(); unresolvedThenShutdown(); }
export function ownedUnresolvedCatch() { assertResult(); unresolvedCatchShutdown(); }
export function ownedUnresolvedFinally() { return main().finally(unresolvedFinallyShutdown); }
export function unresolvedThen() { return unresolvedThenShutdown(); }
export function unresolvedCatch() { return unresolvedCatchShutdown(); }
export function unresolvedFinally() { return unresolvedFinallyShutdown(); }
