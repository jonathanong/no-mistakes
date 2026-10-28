import { read } from '@bench/db';
import sql from 'sql-template-strings';

function named(value: string) {
  return sql`/* fixedHelper */ SELECT ${value}`;
}
read(named('ok'));

function missing() {
  return sql`SELECT 1`;
}
read(missing());

// The exact trusted tag permits the missing-head finding through an opaque suffix.
function opaqueMissing(suffix: unknown) {
  return sql`SELECT 1`.append(suffix);
}
read(opaqueMissing(' WHERE true'));

function customOpaqueMissing(suffix: unknown) {
  return otherTag`SELECT 1`.append(suffix);
}
read(customOpaqueMissing(' WHERE true'));

function unknown(value: string) {
  return sql`/* ${value} */ SELECT 1`;
}
read(unknown('maybe'));
