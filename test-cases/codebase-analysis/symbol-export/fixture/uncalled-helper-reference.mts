import { alpha } from './source.mts';

function helper() {
  return alpha();
}

export function run() {
  const f = helper;
  return f;
}
