import { read } from '@example/db';
// Non-Array prototype properties must not taint intrinsic projections.
const local = { prototype: 'local' };
local.prototype;
const otherGlobal = { Other: { prototype: 'local' } };
otherGlobal.Other.prototype;
otherGlobal['Other']['prototype'];
function ordinaryObject() { return { prototype: 'local' }; }
ordinaryObject().prototype;
export function run() {
  const parts = ['/* intrinsics/pristine */ SELECT 1'];
  parts.push('');
  parts.forEach(() => 0);
  read(parts.map(value => value).join(''));
}
