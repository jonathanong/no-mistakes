import { imported } from './no-object-mutation-computed-dependency';
const moduleRows = { a: [], b: [] };

export function parameter(out: Record<'a' | 'b', number[]>, items: { kind: 'a' | 'b'; v: number }[]) {
  for (const it of items) out[it.kind].push(it.v);
}

export function importedRows(items: { kind: 'a' | 'b'; v: number }[]) {
  for (const it of items) imported[it.kind].push(it.v);
}

export function moduleScope(items: { kind: 'a' | 'b'; v: number }[]) {
  for (const it of items) moduleRows[it.kind].push(it.v);
}

export function sharedChild(rows: number[], key: string) {
  const out = { a: [], b: rows };
  out[key].push(1);
}

export function escapedChild(sink, key: string) {
  const out = { a: [], b: [] };
  sink(out.a);
  out[key].push(1);
}

export function computedEscape(sink, key: string) {
  const out = { a: [], b: [] };
  sink(out[key]);
  out.a.push(1);
  out.b.push(2);
}

export function sharedUnion(rows: number[], flag: boolean, key: string) {
  const out = { a: flag ? [] : rows, b: [] };
  out[key].push(1);
}
