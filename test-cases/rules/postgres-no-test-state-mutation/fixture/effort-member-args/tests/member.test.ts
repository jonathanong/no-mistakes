import { pool as p } from '@example/db';
import db from '@example/db';
import * as ns from '@example/db';
import type { pool as erasedPool } from '@example/db';
import { pool as otherPool } from '@other/db';

p.query(); // OUTSIDE: low member fallback requires an argument
p.query({ ...{ text: 'DROP TABLE app.spread' } }); // ADVISE: spread object
p.query({ 'text': 'DROP TABLE app.quoted_key' }); // ADVISE: quoted text key
p.query({ 0: 'DROP TABLE app.numeric_key' }); // ADVISE: unknown object shape

const t = 'DROP TABLE app.const_sql';
p.query(t); // ADVISE: local const SQL
p.query({ text: t }); // ADVISE: local const text field
p.query('DROP TABLE app.concat_' + 'sql'); // ADVISE: concatenation
p.query(String.raw`DROP TABLE app.raw_sql`); // ADVISE: String.raw
p.query(['DROP TABLE app.joined_sql'].join('')); // ADVISE: joined SQL
p.query(...['DROP TABLE app.spread_sql']); // ADVISE: spread argument
p.query({ ['text']: 'DROP TABLE app.computed_key' }); // ADVISE: computed text key

const p2 = p;
const p3 = p2;
p2.query(t); // ADVISE: local receiver alias
p3.query({ text: t }); // ADVISE: two-hop receiver alias
p3.query(); // OUTSIDE: low alias fallback requires an argument
let letAlias = p;
letAlias.query(t); // ADVISE: let receiver alias
var varAlias = p;
varAlias.query(t); // ADVISE: var receiver alias
const db2 = db;
db2.query(t); // ADVISE: default receiver alias
const ns2 = ns;
ns2.query(t); // ADVISE: namespace receiver alias

const local = { query(_value: unknown) {} };
const localAlias = local;
localAlias.query(t); // OUTSIDE: unrelated local receiver
const otherAlias = otherPool;
otherAlias.query(t); // OUTSIDE: unrelated imported receiver
erasedPool.query(t); // OUTSIDE: erased import is not a runtime receiver
p.execute(t); // OUTSIDE: a different method

function shadow(p: { query(value: unknown): void }) {
  p.query(t); // OUTSIDE: same-spelling shadow is not the imported receiver
}
