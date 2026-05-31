import { alpha } from './source.mts';

function helper() {
  run();
  return alpha();
}

export function run() {
  return helper();
}
