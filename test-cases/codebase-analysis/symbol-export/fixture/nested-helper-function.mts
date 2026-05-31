import { alpha } from './source.mts';

export function run() {
  function helper() {
    return alpha();
  }
  return helper();
}
