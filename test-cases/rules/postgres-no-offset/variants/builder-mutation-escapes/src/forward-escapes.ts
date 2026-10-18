import sql from "sql-template-strings";
import { read } from "@example/db";

// Hoisted declarations and later assignments must retain captured-name escapes.
{
  function get() { return returned; }
  var returned = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  get().append(sql` OFFSET 5`);
  read(returned); // expect: dynamic
}
{
  function mutate(x: any = defaulted) { x.append(sql` OFFSET 5`); }
  var defaulted = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  mutate();
  read(defaulted); // expect: dynamic
}
{
  let q;
  function get() { return q; }
  q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  get().append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  let q;
  function mutate(x: any = q) { x.append(sql` OFFSET 5`); }
  q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  mutate();
  read(q); // expect: dynamic
}
{
  let q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  function get() { return q; }
  q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  get().append(sql` OFFSET 5`);
  read(q); // expect: dynamic
}
{
  function get() { if (runtimeFlag) return conditional; return null; }
  var conditional = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  get()?.append(sql` OFFSET 5`);
  read(conditional); // expect: dynamic
}
{
  function get() { try { return handled; } catch { return null; } }
  var handled = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  get()?.append(sql` OFFSET 5`);
  read(handled); // expect: dynamic
}
{
  function get() {
    switch (runtimeFlag) { case 1: return selected; default: return null; }
  }
  var selected = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  get()?.append(sql` OFFSET 5`);
  read(selected); // expect: dynamic
}
{
  function get() { return [retained]; }
  var retained = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  get()[0].append(sql` OFFSET 5`);
  read(retained); // expect: dynamic
}

// Primitive copies, unrelated objects, and sibling lexical bindings stay known.
{
  function get() { return primitive; }
  const primitive = "SELECT id FROM accounts WHERE active IS TRUE";
  opaque(get());
  read(primitive);
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  read(q);
}
const independent = sql`SELECT id FROM accounts WHERE active IS TRUE`;
read(independent);
