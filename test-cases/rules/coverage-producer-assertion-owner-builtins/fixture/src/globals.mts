import { AsyncLocalStorage } from 'node:async_hooks';
import { join } from 'node:path';
import { owner } from './owner.mjs';
const storage = new AsyncLocalStorage<string>();
export function globalsOwned(items: number[]) {
  parseInt('1');
  join('a', 'b');
  Array.from(items, () => owner());
  storage.exit(() => owner());
  setTimeout(() => owner(), 0);
  Promise.resolve(1).then(() => owner());
}
export function importedCallback() { Promise.resolve(1).then(owner); }
export function memberCallback(items: number[], holder: { cb: (value: number) => void }, pick: () => () => void) {
  items.map(holder.cb);
  items.map(pick());
}
