import { read } from '@example/db';
// Even local namespace properties named Array conservatively count as exposure.
const otherGlobal = { Array: { prototype: 'local' } };
otherGlobal.Array.prototype;
otherGlobal['Array']['prototype'];
export function run() { read(['SELECT 1'].join('')); }
