import sql from "sql-template-strings";
import { read } from "@example/db";

// Container references escape even when no ordinary call receives the builder.
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  const [a] = [q];
  a.append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  const { x } = { x: q };
  x.append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  const o = { q };
  o.q.append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  const o: any = {};
  o.p = q;
  o.p.append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  const bag: any[] = [];
  bag[0] = q;
  bag[0].append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  const bag = [...[q]];
  bag[0].append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  for (const x of [q]) x.append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  const s = new Set([q]);
  for (const x of s) x.append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  function get() { return q; }
  get().append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  function mutate(x: any = q) { x.append(sql` OFFSET 5`); }
  mutate();
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  function* values() { yield q; }
  for (const x of values()) x.append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  const o = { ...{ q } };
  o.q.append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  // A later argument can retain the same mutable object read by the first.
  read(q, [q]); // expect: dynamic
}
{
  function get() { return q; }
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  get().append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  function mutate(x: any = q) { x.append(sql` OFFSET 5`); }
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  mutate();
  read(q); // expect: dynamic
}
{
  const get = () => q;
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  get().append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  q.opaqueMutation();
  read(q); // expect: dynamic
}
