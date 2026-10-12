import { read } from '@data-stores/psql';
import { valkey as cache } from '@data-stores/valkey';
import { lookup } from './helpers.mjs';
const items = [1, 2];
export function importedAndLocal() {
  items.map(read);
  {
    const read = (item: number) => item;
    items.map(read);
  }
}
export function memberAndLocal() {
  items.map(cache.get);
  {
    const cache = { get: (item: number) => item };
    items.map(cache.get);
  }
}
export function helperAndLocal() {
  items.forEach(lookup);
  {
    const lookup = (item: number) => item;
    items.forEach(lookup);
  }
}
// All three callback uses have the same spelling and caller on one source line.
export function repeatedBindings() { items.map(read); { const read = (item: number) => item; items.map(read); } items.map(read); }
// A different iteration construct must not inherit another callback's binding.
export function differentConstructs() { items.map(read); { const read = (item: number) => item; items.forEach(read); } }
