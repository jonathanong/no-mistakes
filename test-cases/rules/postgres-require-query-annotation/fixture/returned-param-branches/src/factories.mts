import sql from 'sql-template-strings'
// This is the existing exact five-statement source grammar, unchanged by this slice.
export function factory(select, extra, from = sql` FROM records r`) {
  const statement = select.append(sql`id`)
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}
export function unproven(select, extra, from = sql` FROM records r`) { return other() }
