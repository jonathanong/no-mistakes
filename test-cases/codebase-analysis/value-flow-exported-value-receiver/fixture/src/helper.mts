import { receiver, unrelated } from './receiver.mts';
export function acquire() { return receiver.get; }
export function acquireSibling() { return receiver.set; }
export function acquireUnrelated() { return unrelated.get; }
