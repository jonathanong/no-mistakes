import { alpha } from './source.mts';

export function run(fn = function (alpha: () => number) {
  return alpha();
}) {
  fn();
  return alpha();
}
