import sql, { type SQLStatement } from 'sql-template-strings';
import { write } from '@app/db';

function project(select: SQLStatement, id: string): SQLStatement {
  const statement = select.append(sql` id`);
  statement.append(sql` FROM items WHERE id = ${id}`);
  return statement;
}

write(project(sql`/* safe */ SELECT`, 'item'));
write(project(sql`SELECT`, 'item')); // Missing annotation after recovery.

function page(select: SQLStatement): SQLStatement {
  select.append(sql` id FROM items OFFSET 2`);
  return select;
}
write(page(sql`/* offset */ SELECT`)); // The ordinary OFFSET diagnostic.

function opaque(select: SQLStatement, tail: string): SQLStatement {
  select.append(tail);
  return select;
}
write(opaque(sql`/* opaque */ SELECT`, ' id FROM items'));
