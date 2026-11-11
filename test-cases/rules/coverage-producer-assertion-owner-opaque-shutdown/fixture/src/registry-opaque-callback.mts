import { shutdownStore } from './opaque-store.mts';

export function thenShutdown() { return Promise.resolve().then(shutdownStore); }
export function catchShutdown() { return Promise.resolve().catch(shutdownStore); }
export function finallyShutdown() { return Promise.resolve().finally(shutdownStore); }
