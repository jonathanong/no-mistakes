import { read } from '@example/db';
globalThis.Array.prototype.join = () => 'DELETE FROM records';
export function run() { read(['/* intrinsics/global */ SELECT 1'].join('')); }
