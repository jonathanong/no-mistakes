import { alpha } from './source.mts';

export function run(alpha: () => number) {
  return alpha();
}
