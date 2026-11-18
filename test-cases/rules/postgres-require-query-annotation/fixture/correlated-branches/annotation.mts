import sql from 'sql-template-strings'
import { read } from './db.mjs'

export function listEither(mode: 'widgets' | 'gadgets' | 'all') {
  const statement = sql`/* listEither */ WITH candidates AS (`
  if (mode === 'widgets' || mode === 'all') {
    statement.append(sql`SELECT id FROM widgets`)
  }
  if (mode === 'all') {
    statement.append(sql` UNION ALL `)
  }
  if (mode === 'gadgets' || mode === 'all') {
    statement.append(sql`SELECT id FROM gadgets`)
  }
  statement.append(sql`) SELECT id FROM candidates LIMIT 5`)
  return read(statement)
}

export function lookup() {
  return read(sql`/* lookup */ SELECT id FROM widgets LIMIT 5`)
}

export function listIndependent(
  left: 'widgets' | 'gadgets',
  right: 'widgets' | 'gadgets',
) {
  const statement = sql`/* listIndependent */ WITH candidates AS (`
  if (left === 'widgets') {
    statement.append(sql`SELECT id FROM widgets`)
  }
  if (right === 'gadgets') {
    statement.append(sql`SELECT id FROM gadgets`)
  }
  statement.append(sql`) SELECT id FROM candidates LIMIT 5`)
  return read(statement)
}

export function listAfterReassignment(mode: 'widgets' | 'gadgets') {
  const statement = sql`/* listAfterReassignment */ WITH candidates AS (`
  if (mode === 'widgets') {
    statement.append(sql`SELECT id FROM widgets`)
  }
  mode = 'gadgets'
  if (mode === 'gadgets') {
    statement.append(sql`SELECT id FROM gadgets`)
  }
  statement.append(sql`) SELECT id FROM candidates LIMIT 5`)
  return read(statement)
}
