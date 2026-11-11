import { shutdownStore } from './shutdown-store.mts';
import { acquirePending } from 'fixture-runtime';

let pending = acquirePending();

export function installPending(value: unknown) {
  pending = value;
}

export function thenShutdown() { return pending.then(shutdownStore); }
export function catchShutdown() { return pending.catch(shutdownStore); }
export function finallyShutdown() { return pending.finally(shutdownStore); }

function getPending() { return pending; }
export function opaqueThenShutdown() { return getPending().then(shutdownStore); }
export function opaqueCatchShutdown() { return getPending().catch(shutdownStore); }
export function opaqueFinallyShutdown() { return getPending().finally(shutdownStore); }
