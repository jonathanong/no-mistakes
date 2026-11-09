import { AsyncLocalStorage as Store } from 'node:async_hooks';

declare function run(value?: unknown): void;

export function callbackFamilies(rows: number[]) {
  rows.map(value => value.toFixed(1));
  'text'.replace('x', run);
  Promise.resolve(1).then(run);
  new Store().run('request', run);
  Array.from(rows, run);
  Buffer.from(rows, run);
  'text'.replace(/x/, 'y');
  Promise.resolve(1).finally(1);

  new Map().then(run);
  (1).map(run);
  ({}).run('request', run);
  const empty = [];
  empty.forEach(run);
  [,,].map(run);
}
