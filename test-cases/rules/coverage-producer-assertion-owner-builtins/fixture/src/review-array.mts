import { owner } from './owner.mjs';
export function sparseFind() { [,].find(owner); }
export function sparseNamed() { const sparse = [,]; Array.from(sparse, owner); }
export function sparseSorted() { [,].toSorted().forEach(owner); }
export function emptySparseSlice() { [,].slice(1).find(owner); }
export function emptySparseSplice() { [,,].splice(0, 0).find(owner); }
export function holeSlice() { [,,].slice(1).find(owner); }
export function mutatedHoles() { const holes = [,]; holes.pop(); holes.find(owner); }
export function lastIndex(rows: number[]) { rows.findLastIndex(owner); }
export function emptyStable() { const empty = []; empty.findLastIndex(owner); }
export function sparseSkip() { const sparse = [,]; sparse.map(owner); }
export function mutated() { const mutatedRows = []; mutatedRows.push(1); mutatedRows.forEach(owner); }
export function aliasMutated() {
  const mutableSource = []; const mutableAlias = mutableSource; mutableAlias.push(1); mutableSource.forEach(owner);
}
export function escaped(mutate: (rows: unknown[]) => void) {
  const escapedRows = []; mutate(escapedRows); escapedRows.forEach(owner);
}
