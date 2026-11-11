import { opaqueThenShutdown, opaqueCatchShutdown, opaqueFinallyShutdown } from './registry.mts';

export function runThen() { return opaqueThenShutdown(); }
export function runCatch() { return opaqueCatchShutdown(); }
export function runFinally() { return opaqueFinallyShutdown(); }

import { thenShutdown, catchShutdown, finallyShutdown } from './registry-opaque-callback.mts';
export function runUnresolvedThen() { return thenShutdown(); }
export function runUnresolvedCatch() { return catchShutdown(); }
export function runUnresolvedFinally() { return finallyShutdown(); }
