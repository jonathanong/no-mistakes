type Numbers = number[];
type Pair = readonly [number, number];
export function alias(rows: Numbers) {
  const ready: number[] = [];
  rows.map(value => { ready.push(value); return value; });
  return ready;
}
export function tuple(rows: Pair) {
  const ready: number[] = [];
  rows.forEach(value => { ready.push(value); });
  return ready;
}
export function member(store: { rows: Numbers }) {
  const ready: number[] = [];
  store.rows.filter(value => { ready.push(value); return true; });
  return ready;
}
export function castAlias(rows: Numbers) {
  const ready: number[] = [];
  (rows as number[]).flatMap(value => { ready.push(value); return [value]; });
  return ready;
}
declare function getRows(): Numbers;
export function returned() {
  const ready: number[] = [];
  getRows().forEach(value => { ready.push(value); });
  return ready;
}
export function union(rows: Numbers | readonly number[]) {
  const ready: number[] = [];
  rows.forEach(value => { ready.push(value); });
  return ready;
}
export function continuing(rows: Numbers, stop: boolean) {
  if (stop) return { rows };
  const ready: number[] = [];
  rows.forEach(value => { ready.push(value); });
  return ready;
}
export function arrayThis(this: number[]) {
  const ready: number[] = [];
  this.forEach(value => { ready.push(value); });
  return ready;
}
