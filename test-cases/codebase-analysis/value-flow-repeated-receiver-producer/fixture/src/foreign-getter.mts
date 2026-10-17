import { foreignGetterReceiver, foreignCopiedReceiver, beforeLoopGetterReceiver } from '@vendor/producer';
// A callable source is evaluated afresh when the caller invokes this getter.
export function acquireForeign() { return foreignGetterReceiver.get; }
// This source was acquired once, before the caller's repeated region.
const copied = foreignCopiedReceiver.get;
export function acquireCopied() { return copied; }
export function acquireSibling() { return foreignGetterReceiver.set; }
export function acquireBeforeLoop() { return beforeLoopGetterReceiver.get; }
