import type { Handler } from './typed-handler.mts';

export function run<Handler>(arg: Handler) {
  return arg;
}
