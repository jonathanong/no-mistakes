import sql from 'sql-template-strings'
import custom from 'custom-sql'
import { read } from '@bench/db'
import { append, extra, unsafe } from './effects.mjs'
import { missing } from './missing.mjs'

export function block(items, tail) {
  const q = sql`/* capturedImport */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  append(q, tail)
  read(q) // Literal
  read(q) // Literal
}
export function multiple(items, tail) {
  const q = sql`/* allEffects */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  append(q, tail)
  extra(q)
  read(q) // Literal
  read(q) // Literal
}
export function paired(items, tail) {
  const q = sql`/* paired */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  append(q, tail)
  read(q.text, q.values) // Literal
}

export function unresolved(items, tail) {
  const q = sql`/* unresolved */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  append(q, tail)
  missing(q)
  read(q) // Unproven
}

export function unsafeOne(items, tail) {
  const q = sql`/* unsafeOne */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  append(q, tail)
  unsafe(q)
  read(q) // Unproven
}

export function wrongTag(items, tail) {
  const q = custom`/* wrongTag */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  append(q, tail)
  read(q) // Unproven
}

export function alias(items, tail) {
  const q = sql`/* alias */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  const alias = q
  append(q, tail)
  read(q) // Unproven
}

export function extraRead(items, tail) {
  const q = sql`/* extraRead */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  outside(q)
  append(q, tail)
  read(q) // Unproven
}

export function expressionArrow(items, tail) {
  const q = sql`/* expressionArrow */ SELECT ${tail}`
  items.forEach(item => q.append(item))
  append(q, tail)
  read(q) // Unproven
}

export function returnedAppend(items, tail) {
  const q = sql`/* returnedAppend */ SELECT ${tail}`
  items.forEach(item => { return q.append(item) })
  append(q, tail)
  read(q) // Unproven
}

export function capturedAlias(items, tail) {
  const q = sql`/* capturedAlias */ SELECT ${tail}`
  items.forEach(item => { return q })
  append(q, tail)
  read(q) // Unproven
}

export function shadow(items, tail) {
  const q = sql`/* shadow */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  { const append = opaque; append(q, tail) }
  read(q) // Unproven
}

export function getterEscape(items, tail) {
  const q = sql`/* getterEscape */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  append(q, { get text() { q.text = 'SELECT unsafe'; return 'suffix' } })
  read(q) // Unproven
}

export function typeUse(items, tail) {
  const q = sql`/* typeUse */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  type Builder = typeof q
  append(q, tail)
  read(q) // Unproven
}

export function budget(items, tail) {
  const q = sql`/* budget */ SELECT ${tail}`
  items.forEach(item => { q.append(item) })
  append(q, tail)
  append(q, tail)
  append(q, tail)
  append(q, tail)
  append(q, tail)
  append(q, tail)
  append(q, tail)
  append(q, tail)
  append(q, tail)
  read(q) // Unproven
}

// Module exports expose the heap independently of these closed local reads.
export const publicBuilder = sql`/* public */ SELECT 1`
items.forEach(item => { publicBuilder.append(item) })
append(publicBuilder, tail)
read(publicBuilder) // Unproven
