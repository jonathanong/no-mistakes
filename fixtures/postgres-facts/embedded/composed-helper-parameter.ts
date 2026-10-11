import sql, { type SQLStatement } from "sql-template-strings";
import { write } from "@example/db";

function project(select: SQLStatement, id: string): SQLStatement {
  const statement = select.append(sql` id`);
  statement.append(sql` FROM items WHERE id = ${id}`);
  return statement;
}
write(project(sql`/* good */ SELECT`, "item"));

function offset(select: SQLStatement): SQLStatement {
  const alias = select;
  alias.append(sql` id FROM items OFFSET 2`);
  return select;
}
write(offset(sql`/* offset */ SELECT`));

function unknown(select: SQLStatement, tail: string): SQLStatement {
  select.append(tail);
  return select;
}
write(unknown(sql`/* unknown */ SELECT`, " id FROM items"));

function injected(select: SQLStatement): SQLStatement {
  select.append(sql` ${select} FROM items`);
  return select;
}
write(injected(sql`/* injected */ SELECT`));

function conditional(select: SQLStatement): SQLStatement {
  if (Math.random()) select.append(sql` OFFSET 1`);
  return select;
}
write(conditional(sql`/* conditional */ SELECT id FROM items`));

function reassign(select: SQLStatement): SQLStatement {
  select = sql`SELECT id FROM items`;
  return select;
}
write(reassign(sql`/* reassign */ SELECT`));

const dynamic = getSql();
write(project(dynamic, "item"));

function shadow(select: SQLStatement, sql: (strings: TemplateStringsArray) => SQLStatement): SQLStatement {
  select.append(sql` id FROM items`);
  return select;
}
write(shadow(sql`/* shadow */ SELECT`, getSql));

function aliasCycle(select: SQLStatement): SQLStatement {
  const first = second;
  const second = first;
  first.append(sql` id FROM items`);
  return select;
}
write(aliasCycle(sql`/* cycle */ SELECT`));

function useAsValue(select: SQLStatement): SQLStatement {
  select.append(sql` ${select} FROM items`);
  return select;
}
write(useAsValue(sql`/* value */ SELECT`));

write(externalHelper(sql`/* external */ SELECT`));

const partial = sql`/* partial */ SELECT`;
partial.append(getTail());
write(project(partial, "item"));

const timed = sql`/* timed */ SELECT`;
write(project(timed, "item"));
timed.append(sql` OFFSET 1`);
write(project(timed, "item"));

function mutate(select: SQLStatement): string {
  select.append(sql` OFFSET 3`);
  return "item";
}
write(project(timed, mutate(timed)));

function sideEffect(select: SQLStatement): SQLStatement {
  select.append(sql` WHERE id = ${getValue()}`);
  return select;
}
write(sideEffect(sql`/* side effect */ SELECT id FROM items`));
write(project("/* cast */ SELECT" as unknown as SQLStatement, "item"));

function directReturn(select: SQLStatement): SQLStatement {
  return select.append(sql` id FROM items`);
}
write(directReturn(sql`/* return */ SELECT`));
