import { read } from './query.mjs';
export function run(items: number[]) {
  return items.map(item => read(item));
}
