import { read } from '@example/db';

declare function unrelated(value: unknown): unknown;

// This payload is intentionally unrelated to configured SQL or executor bindings.
unrelated({
  first: ['one', 'two', 'three'],
  second: { a: 'four', b: 'five', c: 'six' },
  third: ['seven', 'eight', 'nine'],
  fourth: { a: 'ten', b: 'eleven', c: 'twelve' },
  fifth: ['thirteen', 'fourteen', 'fifteen'],
  sixth: { a: 'sixteen', b: 'seventeen', c: 'eighteen' },
  seventh: ['nineteen', 'twenty', 'twenty-one'],
  eighth: { a: 'twenty-two', b: 'twenty-three' },
});

read('SELECT id FROM app.items OFFSET 4'); // configured sink keeps full SQL
unrelated(read); // selected executor forwarding keeps full argument

const local = { sql: 'SELECT id FROM app.items OFFSET 5' };
unrelated(local); // opaque escape keeps its reference
read(local.sql); // escaped local cannot prove SQL

function abrupt() {
  throw unrelated(local); // opaque throw call keeps its skeleton and reference
}
