import { owner } from './owner.mjs';
export function numericMap() { (1).map(owner); }
export function plainRun() { ({}).run('id', owner); }
export function mapThen() { new Map().then(owner); }
export function stringFinally() { ''.finally(owner); }
function noop() {}
export function conditionalCallback(flag) { return Promise.resolve().finally(flag ? owner : noop); }
export function unknownCallback(factory) { return Promise.resolve().finally(factory()); }
export function absentCallback() { return Promise.resolve().finally(undefined); }
export function nonCallableCallback() { return Promise.resolve().finally(1); }
export function replaceLeaf() { return 'x'.replace(/x/, 'y'); }
export function emptyArray() { [].forEach(owner); }
export function emptyAlias() { const values = []; values.map(owner); }
export function emptyChain() { [].map(() => 1).forEach(owner); }
export function arrayJoinMap() { return [1].join(',').map(owner); }
export function promiseSliceFinally() { return Promise.resolve(1).slice().finally(owner); }
export function arrayIteratorMap() { return [1].values().map(owner); }
export function factoryValue() { return Promise.resolve(owner).finally(noop); }
export function chainedOwner() { return Promise.resolve().then(owner).finally(noop); }
export function repeatedOwner() { return Promise.resolve().finally(noop).finally(owner); }
export function repeatedNoOwner() { return Promise.resolve().finally(noop).finally(noop); }
export function sparseFromOwner() { return Array.from([,], owner); }
export function sparseFromChain() { return Array.from([,]).forEach(owner); }
