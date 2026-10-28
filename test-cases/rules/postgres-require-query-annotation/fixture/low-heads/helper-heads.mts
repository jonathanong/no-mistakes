import { read } from '@bench/db';
import sql from 'sql-template-strings';

function fixed(value: string) {
  return sql`/* fixed */ SELECT ${value}`;
}
read(fixed('x')); // low: true

const arrow = (value: string) => {
  const suffix = value;
  return `/* arrow */ SELECT ${suffix}`;
};
read(arrow('x')); // low: true

function opaque(suffix: unknown) {
  return sql`/* opaque */ SELECT 1`.append(suffix);
}
read(opaque(' WHERE true')); // low: true

// The opaque suffix preserves a complete missing head only for the trusted tag.
function opaqueMissing(suffix: unknown) {
  return sql`SELECT 1`.append(suffix);
}
read(opaqueMissing(' WHERE true')); // low: false

function customOpaqueMissing(suffix: unknown) {
  return otherTag`SELECT 1`.append(suffix);
}
read(customOpaqueMissing(' WHERE true')); // low: false

function missing() {
  return sql`SELECT 1`;
}
read(missing()); // low: false

function partial(value: string) {
  return `/* ${value} */ SELECT 1`;
}
read(partial('name')); // low: false

function early(flag: boolean) {
  if (flag) return sql`/* early */ SELECT 1`;
  return sql`/* early */ SELECT 2`;
}
read(early(true)); // low: false

async function asynchronous() {
  return sql`/* async */ SELECT 1`;
}
read(asynchronous()); // low: false

function* generated() {
  return sql`/* generated */ SELECT 1`;
}
read(generated()); // low: false

function mutable() {
  return sql`/* mutable */ SELECT 1`;
}
mutable = () => sql`SELECT 1`;
read(mutable()); // low: false

function custom() {
  return otherTag`/* custom */ SELECT 1`;
}
read(custom()); // low: false

function named(value: string) {
  return sql`/* ${value} */ SELECT 1`;
}
read(named('x')); // low: false

read(fixed?.('x')); // low: false
read(fixed(...parts)); // low: false
{
  const fixed = () => sql`SELECT 1`;
  read(fixed()); // low: false
}
