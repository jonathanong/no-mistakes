import { alpha, beta } from './source.mts';

export function run() {
  function alpha() {
    return beta();
  }
  return alpha();
}
