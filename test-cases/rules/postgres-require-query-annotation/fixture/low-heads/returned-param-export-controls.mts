import sql from 'sql-template-strings'
import other from './custom.mjs'
// Exact source identity and dynamic-scope controls accompany the positive factory.
export function accepted(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

export function untrustedDefault(select, extra, from = other` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

export function dynamicScope(select, extra, from = sql` FROM records`) {
  const statement = select.append(arguments[0])
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

export function changed(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

changed = opaque
