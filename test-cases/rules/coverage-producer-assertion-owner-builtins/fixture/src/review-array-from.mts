import { owner } from './owner.mjs';

export function emptyFrom() { Array.from([]).forEach(owner); }
export function sparsePositive() { Array.from([,,]).forEach(owner); }
export function sparseFrom(start: number) { Array.from([,,].slice(start)).forEach(owner); }
export function mutatedSparseFrom(start: number) {
  const sparse = [,,].slice(start);
  sparse.push(undefined);
  Array.from(sparse).forEach(owner);
}
