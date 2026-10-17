import { read, sql } from '@example/db';
export function trueDefault() { read('/* booleans/true */ SELECT ' + [true].join()); }
export function falseDefault() { read('/* booleans/false */ SELECT ' + [false].join()); }
export function pair() { read('/* booleans/pair */ SELECT ' + [true, false].join(', ')); }
export function mixed() { read('/* booleans/mixed */ SELECT ' + [true, 1, 'value', false].join(', ')); }
export function template() { read(`/* booleans/template */ SELECT ${true}, ${false}`); }
export function concatenate() { read('/* booleans/concat */ SELECT ' + true + ', ' + false); }
export function leftOperand() { read('/* booleans/left */ SELECT ' + (true + '') + ', ' + (false + '')); }
export function conditional(flag: boolean) { read('/* booleans/conditional */ SELECT ' + [flag ? true : false].join('')); }
export function boundValue(value: unknown) {
  const query = sql`/* booleans/bind */ SELECT ${value}, `;
  query.append([true, false].join(', '));
  read(query);
}
// A trusted-tag interpolation stays a generated bind, not authored boolean text.
export function taggedBooleans() { read(sql`/* booleans/tagged */ SELECT ${true}, ${false}`); }
export function unknownValue(value: unknown) { read('/* booleans/unknown */ SELECT ' + [value].join()); }
// Object coercion may execute arbitrary code and must remain unclassified.
export function opaqueObject() { read('/* booleans/object */ SELECT ' + [{ toString() { return 'true'; } }].join()); }
// Boolean arithmetic is not string concatenation and stays outside this recovery.
export function arithmetic() { read('/* booleans/arithmetic */ SELECT ' + (true + false)); }
