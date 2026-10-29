import sql from 'sql-template-strings'

// The one-argument caller never enters either extra branch.
export function selectWithOwner(select, extra, from = sql` FROM records r`) {
  const statement = select.append(sql` r.id`).append(', r.name').append(sql` AS name`)
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

export function wrongReturn(select, extra, from = sql` FROM records r`) {
  const statement = select.append(sql` r.id`)
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  if (extra) return sql`/* other */ SELECT 1`
  return statement
}

export function aliasEscape(select, extra, from = sql` FROM records r`) {
  const statement = select.append(sql` r.id`)
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  expose(statement)
  return statement
}

export function unsafeDefault(select, extra, from = mutate(select)) {
  const statement = select.append(sql` r.id`)
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

// An empty native default is inert, but the first append can complete a head.
export function continueHead(select, extra, from = sql``) {
  const statement = select.append(sql`NING SELECT 1`)
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

// An empty native default is inert, but the first append can complete a head.
export function closeComment(select, extra, from = sql``) {
  const statement = select.append(sql` repaired */ SELECT 1`)
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}
