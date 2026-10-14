import { read } from '@example/db';
function fill(parts: string[], flag: boolean) {
  if (flag) parts.push('a');
  else parts.push('a', 'b');
}
export function divergentShape(flag: boolean) {
  const parts: string[] = [];
  fill(parts, flag);
  // The joined storage sentinel is not a one-element runtime array.
  if (parts.length === 1) read('/* shapes/divergent */ SELECT 1');
  else read('/* shapes/divergent */ DELETE FROM records');
}
export function denseShape() {
  const parts = ['a', 'b'];
  if (parts.length === 2) read('/* shapes/dense */ SELECT 2');
  else read('/* shapes/dense */ DELETE FROM records');
}
