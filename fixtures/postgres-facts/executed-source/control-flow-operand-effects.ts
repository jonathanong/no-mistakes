import { read, sql } from '@example/db';
export function joinArgument() {
  const query = sql`/* operands/join-argument */ SELECT 1`;
  const parts = ['1'];
  const result = parts.join((opaque(), ', '));
  read(query);
}
export function joinReceiver() {
  const query = sql`/* operands/join-receiver */ SELECT 1`;
  const parts = ['1'];
  const result = (opaque(), parts).join(', ');
  read(query);
}
export function pushArgument() {
  const query = sql`/* operands/push */ SELECT 1`;
  const parts = [];
  parts.push((opaque(), '1'));
  read(query);
}
export function mapArgument() {
  const query = sql`/* operands/map */ SELECT 1`;
  const parts = ['1'];
  const result = parts.map((opaque(), value => value));
  read(query);
}
export function forEachArgument() {
  const query = sql`/* operands/for-each */ SELECT 1`;
  ['1'].forEach((opaque(), () => {}));
  read(query);
}
export function stringBinary() {
  const query = sql`/* operands/string-binary */ SELECT 1`;
  const result = '' + (opaque(), 'x');
  read(query);
}
export function numberBinary() {
  const query = sql`/* operands/number-binary */ SELECT 1`;
  const result = (opaque(), 1) + 2;
  read(query);
}
function appendKnown(query: unknown) { query.append(' WHERE true'); }
export function forwardedArgument() {
  const query = sql`/* operands/forwarded */ SELECT 1`;
  appendKnown((opaque(), query));
  read(query);
}
export function appendReceiver() {
  const query = sql`/* operands/append-receiver */ SELECT 1`;
  const result = (opaque(), query).append(' WHERE true');
  read(query);
}
export function appendTail() {
  const query = sql`/* operands/append-tail */ SELECT 1`;
  query.append((opaque(), ' WHERE true'));
  read(query);
}
export function knownOperands() {
  const query = sql`/* operands/known */ SELECT 1`;
  const parts = ['first AS (SELECT 1)'];
  const statement = `/* operands/collection */ WITH ${parts.map(value => value).join(', ')} SELECT * FROM first`;
  read(statement);
  read(query);
}
export function opaqueCoercion() {
  const query = sql`/* operands/coercion */ SELECT 1`;
  const value = { toString() { query.append(' UPDATE records SET id = id'); return 'x'; } };
  const result = '' + value;
  read(query);
}
