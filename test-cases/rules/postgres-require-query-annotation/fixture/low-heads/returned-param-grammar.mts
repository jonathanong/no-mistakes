import sql from 'sql-template-strings'
// Every rejected declaration violates a distinct bounded identity condition.

function accepted(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badParameterPattern({select}, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badFirstDefault(select = opaque(), extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badSecondDefault(select, extra = {}, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badDefaultKind(select, extra, from = opaque()) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badDefaultHole(select, extra, from = sql` FROM ${opaque()}`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badDefaultTag(select, extra, from = ns.sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badDeclarationKind(select, extra, from = sql` FROM records`) {
  let statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badAliasPattern(select, extra, from = sql` FROM records`) {
  const { statement } = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badParameterRoot(select, extra, from = sql` FROM records`) {
  const statement = other.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badBareRoot(select, extra, from = sql` FROM records`) {
  const statement = select
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badAppendDepth(select, extra, from = sql` FROM records`) {
  const statement = select.append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badRootObject(select, extra, from = sql` FROM records`) {
  const statement = ({}).append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badGuardStatement(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badGuardElse(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns); else opaque()
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badGuardExpression(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra.enabled) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badGuardName(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (other) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badGuardBlock(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) { statement.append(extra.columns) }
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badFromExpression(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  0
  if (extra) statement.append(extra.join)
  return statement
}

function badFromReceiver(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  object.query.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badFromAlias(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  other.append(from)
  if (extra) statement.append(extra.join)
  return statement
}

function badFromArgument(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append('from')
  if (extra) statement.append(extra.join)
  return statement
}

function badFromName(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(other)
  if (extra) statement.append(extra.join)
  return statement
}

function badReturnExpression(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return other()
}

function badReturnName(select, extra, from = sql` FROM records`) {
  const statement = select.append(' columns')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return other
}
