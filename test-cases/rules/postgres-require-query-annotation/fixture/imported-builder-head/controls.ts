import sql from 'sql-template-strings'
import { read } from 'db'
import { selectWithOwner } from './select-with-owner.ts'
import { strings, dynamic, forwarded } from './controls-helpers.ts'

export function nested(first: boolean, second: boolean) {
  const query = first
    ? strings(sql`/* first */ SELECT `)
    : second ? selectWithOwner(sql`/* second */ SELECT `) : strings(sql`/* third */ SELECT `)
  query.append(' LIMIT 1')
  return read(query.text, query.values)
}
export function missing(first: boolean) {
  const query = first ? strings(sql`SELECT `) : selectWithOwner(sql`SELECT `)
  return read(query.text, query.values)
}
export function mixed(first: boolean) {
  const query = first ? strings(sql`/* named */ SELECT `) : strings(sql`SELECT `)
  return read(query.text, query.values)
}
export function unprovenArm(first: boolean) {
  const query = first ? strings(sql`/* named */ SELECT `) : dynamic(sql`/* named */ SELECT `)
  return read(query.text, query.values)
}
export function secondHop() {
  const query = forwarded(sql`/* named */ SELECT `)
  return read(query.text, query.values)
}
export function escaped() {
  const query = strings(sql`/* named */ SELECT `)
  consume(query)
  return read(query.text, query.values)
}
export function overwritten() {
  const query = strings(sql`/* named */ SELECT `)
  query.text = 'SELECT 1'
  return read(query.text, query.values)
}
export function captured() {
  const query = strings(sql`/* named */ SELECT `)
  const closure = () => query.append(' LIMIT 1')
  closure()
  return read(query.text, query.values)
}
export function mutable() {
  let query = strings(sql`/* named */ SELECT `)
  query = opaque()
  return read(query.text, query.values)
}
export function opaqueSuffix() {
  const query = strings(sql`/* named */ SELECT `)
  query.append(opaque())
  return read(query.text, query.values)
}
export function repeated() {
  const query = strings(sql`/* named */ SELECT `)
  for (const id of ids) read(query.text, query.values)
}
export function alias() {
  const query = strings(sql`/* named */ SELECT `)
  const copy = query
  return read(query.text, query.values)
}
export function replacedAppend() {
  const query = strings(sql`/* named */ SELECT `)
  query.append = opaque
  query.append(' LIMIT 1')
  return read(query.text, query.values)
}
export function untrustedTag(first: boolean, custom: typeof sql) {
  const query = first ? strings(sql`/* named */ SELECT `) : strings(custom`/* named */ SELECT `)
  return read(query.text, query.values)
}
