import { foreignSnapshot, foreignLive, foreignAfter } from '@vendor/valkey';
const saved = foreignSnapshot.get;
export function acquireCopied(fn = saved) { return fn; }
export function acquireLive(fn = foreignLive.get) { return fn; }
function replacement() {}
export function mutateForeign() { foreignLive.get = replacement; }
foreignAfter.get = replacement;
const afterWrite = foreignAfter.get;
export function acquireAfter(fn = afterWrite) { return fn; }
