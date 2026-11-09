import { owner } from './owner.mjs';
type Promise = { finally(callback: () => void): void };
type Array<T> = { map(callback: () => void): T };
type AsyncLocalStorage = { run(value: string, callback: () => void): void };
export function promiseType(value: Promise) { value.finally(owner); }
export function arrayType(value: Array<void>) { value.map(owner); }
export function storageType(value: AsyncLocalStorage) { value.run('request', owner); }
