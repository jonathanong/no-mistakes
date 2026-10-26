declare const rows: Record<string, { x: number }>;
Object.entries(rows).find(([, value]) => value.x === 1);
const [first, , third] = [1, , 3];
const [, [, nested], ...rest] = [, [, 2], , 4];
const sparse = [, { x: 1 }, ,];
const { entry: [, item] = [, { x: 1 }] } = {};
function optionalChildren() {
  let uninitialized;
  if (uninitialized) return;
}
