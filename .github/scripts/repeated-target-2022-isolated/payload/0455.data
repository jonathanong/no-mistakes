import { read } from '@example/db';
function fill(parts: string[], flag: boolean) {
  if (flag) parts.push('a');
  else parts.push('a', 'b');
}
function build(flag: boolean) {
  const parts: string[] = [];
  fill(parts, flag);
  // The helper's joined storage sentinel is not a one-element runtime array.
  if (parts.length === 1) return '/* shapes/divergent */ SELECT 1';
  return '/* shapes/divergent */ DELETE FROM records';
}
export function divergentShape(flag: boolean) {
  // Both runtime statements reach this single executor site.
  read(build(flag));
}
function denseBuild() {
  const parts = ['a', 'b'];
  if (parts.length === 2) return '/* shapes/dense */ SELECT 2';
  return '/* shapes/dense */ DELETE FROM records';
}
export function denseShape() {
  read(denseBuild());
}
