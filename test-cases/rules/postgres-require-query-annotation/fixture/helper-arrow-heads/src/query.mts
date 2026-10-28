import { read } from '@bench/db';
import sql from 'sql-template-strings';
import { sql as custom } from '@example/custom';

const literal = () => '/* literal */ SELECT 1';
read(literal()); // low: true
const template = (value: string) => `/* template */ SELECT ${value}`;
read(template('x')); // low: true
const tagged = (value: string) => sql`/* tagged */ SELECT ${value}`;
read(tagged('x')); // low: true
const wrapped = () => (sql`/* wrapped */ SELECT 1` as unknown);
read(wrapped()); // low: true
const appended = (tail: unknown) => sql`/* append */ SELECT 1`.append(tail);
read(appended(dynamic)); // low: true
const missing = () => sql`SELECT 1`;
read(missing()); // low: false
const opaqueMissing = (tail: unknown) => sql`SELECT 1`.append(tail);
read(opaqueMissing(dynamic)); // low: false
const literalMissing = () => 'SELECT 1';
read(literalMissing()); // low: false
const partial = (name: string) => sql`/* ${name} */ SELECT 1`;
read(partial('x')); // low: false
const asynchronous = async () => sql`/* async */ SELECT 1`;
read(asynchronous()); // low: false
const untrusted = () => custom`/* custom */ SELECT 1`;
read(untrusted()); // low: false
const untrustedAppend = (tail: unknown) => custom`SELECT 1`.append(tail);
read(untrustedAppend(dynamic)); // low: false
const captured = sql`/* captured */ SELECT 1`;
const capture = () => captured;
read(capture()); // low: false
const multiple = (flag: boolean) => flag ? sql`/* a */ SELECT 1` : sql`/* b */ SELECT 2`;
read(multiple(true)); // low: false
const rebound = () => sql`/* rebound */ SELECT 1`;
rebound = () => sql`SELECT 1`;
read(rebound()); // low: false
{
  const sql = custom;
  const shadowed = () => sql`/* shadowed */ SELECT 1`;
  read(shadowed()); // low: false
}
read(tagged?.('x')); // low: false
read(tagged(...parts)); // low: false
