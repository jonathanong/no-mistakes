type Opaque = { trim(): string };
export function typed(
  flag: boolean, whole: bigint, literal: 'x', numberLiteral: 1, booleanLiteral: true,
  tuple: [number, string], textual: ('a' | 'b')[], mixed: (string | number)[],
  arrays: (string[] | number[])[], unknown: (string | Opaque)[],
  readonlyRows: readonly number[], genericRows: Array<string>,
  genericReadonly: ReadonlyArray<number>, nestedReadonly: (readonly number[])[],
) {
  flag.toString(); whole.toString(); literal.trim(); numberLiteral.toFixed(1);
  booleanLiteral.valueOf(); tuple.join(',');
  textual.map(t => t.trim());
  mixed.map(m => m.trim());
  arrays.forEach(a => a.join(','));
  unknown.forEach(u => u.trim());
  readonlyRows.forEach(n => n.toFixed(1));
  genericRows.forEach(g => g.trim());
  genericReadonly.forEach(p => p.toFixed(1));
  nestedReadonly.forEach(a => a.join(','));
}
