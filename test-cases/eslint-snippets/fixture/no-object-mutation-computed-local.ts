export function computedVar(items: { kind: 'a' | 'b'; v: number }[]) {
  const out: Record<'a' | 'b', number[]> = { a: [], b: [] };
  for (const it of items) out[it.kind].push(it.v);
  return out;
}

export function dotted(items: { kind: 'a' | 'b'; v: number }[]) {
  const out: Record<'a' | 'b', number[]> = { a: [], b: [] };
  for (const it of items) out.a.push(it.v);
  return out;
}

export function computedLiteral(items: { kind: 'a' | 'b'; v: number }[]) {
  const out: Record<'a' | 'b', number[]> = { a: [], b: [] };
  for (const it of items) out['a'].push(it.v);
  return out;
}

export function computedAssignment(items: { kind: 'a' | 'b'; v: number[] }[]) {
  const out: Record<'a' | 'b', number[]> = { a: [], b: [] };
  for (const it of items) out[it.kind] = it.v;
  return out;
}
