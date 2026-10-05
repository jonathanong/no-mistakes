// Unsupported entries must fail closed alongside the known root destination.
export const pair = ['/', '/'] as const;
export function irrelevant() {}
const base = { destination: '/' };
const object = { nested: base };
const alias = base;
unknown(alias);
unknown(() => object);
unknown(function () { return object; });
const sparse = [, '/'];
const [first, , ...rest] = pair;
const { missing, ...others } = object;
const [fallback = '/'] = unknown();
declare const uninitialized: string;
export default {
 redirects: () => [
  { destination: '/' },
  ...[{ ...{ destination: '/' } }],
  { ...unknown(), destination: '/' },
  { destination: '/', ...unknown() },
  { [unknown()]: '/' },
  { destination() { return '/'; } },
  { 42: '/', destination: '/' },
  {},
  { destination: object.missing },
  { destination: unknown().missing },
  { destination: pair['0'] },
  { destination: object[0] },
  { destination: pair[0.5] },
  { destination: pair[99] },
  { destination: sparse[0] },
  ...pair.map(),
  ...pair.map(...unknown()),
  ...pair.map(function (value) { return { destination: value }; }),
  ...pair.map(async value => ({ destination: value })),
  ...pair.map((value, index) => ({ destination: value })),
  ...pair.map((...values) => ({ destination: values[0] })),
  ...unknown().map(value => ({ destination: value })),
  ...pair.other(value => ({ destination: value })),
  ...pair.map?.(value => ({ destination: value })),
  ...pair?.map(value => ({ destination: value })),
  ...pair.map(value => { ; return { destination: value }; }),
  ...pair.map(value => { return; }),
  ...pair.map(value => { if (unknown()) return { destination: value }; }),
  ...pair.map(([value]) => ({ destination: value })),
  ...pair.map(({ value }) => ({ destination: value })),
  ...pair.map((value = '/') => ({ destination: value })),
  { destination: fallback },
  { destination: `${unknown()}` },
 ],
};
