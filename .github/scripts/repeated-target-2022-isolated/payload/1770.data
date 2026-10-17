import { read } from '@example/db';
// Even a read can hand the prototype to opaque code; this static guard is conservative.
const prototype = Array.prototype;
export function run() { read(['/* intrinsics/exposure */ SELECT 1'].join('')); }
