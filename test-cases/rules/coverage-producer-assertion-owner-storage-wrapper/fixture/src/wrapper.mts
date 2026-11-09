import { AsyncLocalStorage } from 'node:async_hooks';
const storage = new AsyncLocalStorage<string>();
export function withOwner<Result>(handler: () => Result): Result {
  return storage.run('id', handler);
}
export function unresolved<Result>(handler: () => Result): Result {
  return storage.run('id', handler);
}
export function withNoOwner<Result>(handler: () => Result): Result {
  return storage.run('id', handler);
}
export function withNamespaceOnly<Result>(handler: () => Result): Result {
  return storage.run('id', handler);
}
