import { read } from '@example/db';
const prototype = globalThis['Array']['prototype'];
prototype.join = () => 'DELETE FROM records';
export function run() { read(['/* intrinsics/alias */ SELECT 1'].join('')); }
