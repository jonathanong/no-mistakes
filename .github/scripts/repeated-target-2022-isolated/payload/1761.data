import { read } from '@example/db';
function ordinaryObject() { return { prototype: 'local' }; }
// This call/accessor effect is currently unmodeled even though its source is inert.
ordinaryObject().prototype;
export function run() { read(['SELECT 1'].join('')); }
