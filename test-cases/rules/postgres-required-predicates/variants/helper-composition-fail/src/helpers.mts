import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

const VIEW_READ = " EXISTS (SELECT 1 FROM view_active_plans v WHERE v.tier = 'a')";

// Mutual recursion never bottoms out.
export function ping(): SQLStatement {
  return sql`/* ping */ `.append(pong());
}
function pong(): SQLStatement {
  return sql`/* pong */ `.append(ping());
}

// A loop over a parameter could run any number of times.
export function overParameter(tables: string[]): SQLStatement {
  const statement = sql`SELECT 1 FROM view_active_plans v WHERE v.tier = 'a'`;
  for (const table of tables) {
    statement.append(sql` AND v.id IN (SELECT id FROM `).append(table).append(sql`)`);
  }
  return statement;
}

export function whileLoop(): SQLStatement {
  const statement = sql`SELECT 1 FROM view_active_plans v WHERE v.tier = 'a'`;
  let count = 0;
  while (count < 2) {
    statement.append(sql` OR TRUE`);
    count += 1;
  }
  return statement;
}

// Five independent unknown tests are 32 alternatives: past the cap of 16.
export function manyAlternatives(options: Record<string, boolean>): SQLStatement {
  const statement = sql`SELECT 1 FROM view_active_plans v WHERE v.tier = 'a'`;
  if (options.a) statement.append(sql` AND v.a = 1`);
  if (options.b) statement.append(sql` AND v.b = 1`);
  if (options.c) statement.append(sql` AND v.c = 1`);
  if (options.d) statement.append(sql` AND v.d = 1`);
  if (options.e) statement.append(sql` AND v.e = 1`);
  return statement;
}

// A parameter that is not annotated as a SQLStatement never receives a builder.
export function untypedBuilder(target) {
  target.append(sql` AND EXISTS (SELECT 1 FROM view_active_plans v WHERE v.tier = 'a')`);
  return target;
}

// A name the module does not declare could be anything in a value position.
export function undeclaredCallee(): SQLStatement {
  return sql`SELECT 1 FROM view_active_plans v WHERE v.tier = `.append(mystery());
}

// A parameter called like a function is not the module's helper of that name.
export function throughParameter(build: () => SQLStatement): SQLStatement {
  return sql`SELECT 1 FROM view_active_plans v WHERE v.tier = 'a' AND `.append(build());
}

// Raw text must be a literal at the call site.
export function rawIdentifier(column: string): SQLStatement {
  return sql`SELECT 1 FROM view_active_plans v WHERE v.tier = 'a' AND `.append(`${column} IS NULL`);
}

// A local that rebinds the tag is not the trusted SQL tag.
export function shadowedTag(sql: (strings: TemplateStringsArray) => SQLStatement): SQLStatement {
  return sql`SELECT 1 FROM view_active_plans v WHERE v.tier = 'a'`;
}

// A local that rebinds the tag in the body is not the trusted SQL tag either.
export function localTag(make: (strings: TemplateStringsArray) => SQLStatement): SQLStatement {
  const sql = make;
  return sql`SELECT 1 FROM view_active_plans v WHERE v.tier = 'a'`;
}

// A function with no SQL of its own, handed the caller's builder, may have
// changed it, so the unresolved call cannot be ignored.
export function handsOverBuilder(target: SQLStatement): SQLStatement {
  mutateSomehow(target);
  return target.append(sql` AND EXISTS (SELECT 1 FROM view_active_plans v WHERE v.tier = 'a')`);
}

export function viewRead(): string {
  return VIEW_READ;
}
