import { query, sql } from 'db';
function predicate(value: string) { return sql`${value} = fn(id, token)`; }
query(sql`SELECT ${1}, ${2}, ${3}, ${4}, ${5}, ${6}, ${7}, ${8}, ${9} FROM accounts WHERE `
  .append(predicate('first')).append(sql` AND `).append(predicate('second')));
