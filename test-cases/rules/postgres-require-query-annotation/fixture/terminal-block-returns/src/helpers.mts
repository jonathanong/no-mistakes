import sql from 'sql-template-strings'
import otherSql from '@other/sql'

export function named(flag, options) {
  if (flag) {
    const query = sql`/* block/early */ SELECT id FROM posts WHERE owner = ${options.owner}`
    if (options.cursor) query.append(sql` AND id > ${options.cursor}`)
    query.append(sql` ORDER BY id LIMIT ${options.limit}`)
    return query
  }
  // Two additional native fragment roots contribute suffixes, never the head.
  const first = sql`SELECT id FROM first_posts`
  const second = sql`SELECT id FROM second_posts`
  if (options.cursor) first.append(sql` WHERE id > ${options.cursor}`)
  const query = sql`/* block/final */ WITH selected AS (`
  query.append(first)
  query.append(sql` UNION ALL `)
  query.append(second)
  query.append(sql`) SELECT * FROM selected`)
  return query
}
export function differentNames(flag) {
  if (flag) {
    const early = sql`/* differentlyNamedEarly */ SELECT 1`
    early.append(' LIMIT 1')
    return early
  }
  const final = sql`/* differentlyNamedFinal */ SELECT 2`
  return final
}
export function missing(flag) {
  if (flag) {
    const value = 1
    log(value)
    return sql`SELECT 1`
  }
  return sql`SELECT 2`
}
export function mixed(flag) {
  if (flag) {
    const query = sql`/* mixed */ SELECT 1`
    query.append(' LIMIT 1')
    return query
  }
  return sql`SELECT 2`
}
export function unknown(flag, query) {
  if (flag) {
    const early = sql`/* unknown */ SELECT 1`
    early.append(' LIMIT 1')
    return early
  }
  return query
}
export function aliased(flag) {
  if (flag) {
    const query = sql`/* aliased */ SELECT 1`
    const alias = query
    consume(alias)
    return query
  }
  return sql`/* aliased/final */ SELECT 2`
}
export function mutation(flag) {
  if (flag) {
    const query = sql`/* mutation */ SELECT 1`
    query.text = 'SELECT 2'
    return query
  }
  return sql`/* mutation/final */ SELECT 2`
}
export function capture(flag) {
  if (flag) {
    const query = sql`/* capture */ SELECT 1`
    function overwrite() { query.text = 'SELECT 2' }
    overwrite()
    return query
  }
  return sql`/* capture/final */ SELECT 2`
}
export function hiddenReturn(flag, hidden) {
  if (flag) {
    if (hidden) return sql`SELECT 0`
    const query = sql`/* hidden */ SELECT 1`
    query.append(' LIMIT 1')
    return query
  }
  return sql`/* hidden/final */ SELECT 2`
}
export function nestedBranch(flag, inner) {
  if (flag) {
    log(inner)
    if (inner) return sql`/* nested */ SELECT 1`
  }
  return sql`/* nested/final */ SELECT 2`
}
export function throwsBeforeReturn(flag, abort) {
  if (flag) {
    if (abort) throw Error('no query')
    const query = sql`/* throws */ SELECT 1`
    query.append(' LIMIT 1')
    return query
  }
  return sql`/* throws/final */ SELECT 2`
}
export function independentNestedReturn(flag) {
  if (flag) {
    // Nested function returns are outside the owning function's total census.
    function unrelated() { return 'SELECT 0' }
    const query = sql`/* independent */ SELECT 1`
    query.append(' LIMIT 1')
    return query
  }
  return sql`/* independent/final */ SELECT 2`
}
export function differentOwner(flag, query) {
  if (flag) {
    const query = sql`/* exact owner */ SELECT 1`
    query.append(' LIMIT 1')
    return query
  }
  return query
}
export function fallthrough(flag) {
  if (flag) {
    const query = sql`/* not returned */ SELECT 1`
    query.append(' LIMIT 1')
  }
}
export function untrusted(flag) {
  if (flag) {
    const query = otherSql`/* untrusted */ SELECT 1`
    query.append(' LIMIT 1')
    return query
  }
  return sql`/* trusted/final */ SELECT 2`
}
function abrupt(_value) { throw Error('unsupported') }
export function switched(value) {
  switch (value) {
    case 'a': return sql`/* switch/a */ SELECT 1`
    case 'b': return sql`/* switch/b */ SELECT 2`
    default: return abrupt(value)
  }
}
