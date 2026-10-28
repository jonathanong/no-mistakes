import SQL from 'sql-template-strings';
import { read } from '@example/db';
import { assertNever, returnsInstead, effectBeforeThrow, escapeBeforeThrow, coercingThrow } from './switch-abrupt-helper.mjs';

function selected(mode: string) {
  switch (mode) {
    case 'a':
      return SQL`SELECT 1 OFFSET 1`;
    case 'b':
    case 'c':
      return SQL`SELECT 1`;
    default:
      return assertNever(mode);
  }
}
function unsafeDefault(mode: string) {
  switch (mode) {
    case 'a':
      return SQL`SELECT 1`;
    default:
      return returnsInstead(mode);
  }
}
function unsafeArm(mode: string) {
  switch (mode) {
    case 'a':
      return SQL`SELECT ${mode}`;
    default:
      return assertNever(mode);
  }
}
function unsafeEffect(mode: string) {
  switch (mode) {
    case 'a': return SQL`SELECT 1`;
    default: return effectBeforeThrow(mode);
  }
}
function unsafeEscape(mode: string) {
  switch (mode) {
    case 'a': return SQL`SELECT 1`;
    default: return escapeBeforeThrow(mode);
  }
}
function unsafeCoercion(mode: string) {
  switch (mode) {
    case 'a': return SQL`SELECT 1`;
    default: return coercingThrow(mode);
  }
}
function correlated(mode: string) {
  const selected = mode === 'a';
  if (selected) return SQL`SELECT 1 OFFSET 2`;
  return SQL`SELECT 1`;
}
export function samples(mode: string) {
  read(selected(mode)); // COMPLETE: two returned SQL variants; unknown mode throws.
  read(unsafeDefault(mode)); // RESIDUAL: source-visible default returns.
  read(unsafeArm(mode)); // RESIDUAL: returned interpolation is not literal SQL.
  read(unsafeEffect(mode)); // RESIDUAL: throw argument has a SQL call.
  read(unsafeEscape(mode)); // COMPLETE: source-visible throw cannot return an operand.
  read(unsafeCoercion(mode)); // COMPLETE: coercion cannot return past the throw.
  read(correlated(mode)); // COMPLETE: local boolean preserves both branches.
}
