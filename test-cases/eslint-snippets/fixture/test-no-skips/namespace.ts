import * as v from 'vitest';
v.it['skip']('computed', () => {});
v.describe.only('focus', () => {});
v.test.each([1]).runIf(true)('conditional', () => {});
v.test('ctx', ctx => (ctx as any).skip());
function shadow(v) { v.test.skip(); }
