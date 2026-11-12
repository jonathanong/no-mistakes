import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { appendTypeFilter } from './i-type-filter.mts'
export const unrelated = 1
export function exportedFactory() {
  return sql`/* exportedFactory */ SELECT 1`
}
function bare() {
  return sql`SELECT 1`
}
function overloaded(a: string): ReturnType<typeof sql>
function overloaded(a: number): ReturnType<typeof sql>
function overloaded(a: any) {
  return sql`/* overloaded */ SELECT 1`
}
function reassigned() {
  return sql`/* reassigned */ SELECT 1`
}
reassigned = () => sql`SELECT 2`
function startFactory() {
  return sql`/* startFactory */ SELECT 1`
}
export async function exported(user, type) {
  const q = exportedFactory()
  appendTypeFilter(q, user, type)
  await read(q)
}
export async function unannotated(user, type) {
  const q = bare()
  appendTypeFilter(q, user, type)
  await read(q)
}
export async function overload(user, type) {
  const q = overloaded(1)
  appendTypeFilter(q, user, type)
  await read(q)
}
export async function written(user, type) {
  const q = reassigned()
  appendTypeFilter(q, user, type)
  await read(q)
}
export async function shadowed(startFactory, user, type) {
  const q = startFactory()
  appendTypeFilter(q, user, type)
  await read(q)
}
export async function opaqueEffect(user, opaque) {
  const q = startFactory()
  opaque(q)
  await read(q)
}
function customFactory() {
  return customSql`/* customFactory */ SELECT 1`
}
export async function untrustedTag(user, type) {
  const q = customFactory()
  appendTypeFilter(q, user, type)
  await read(q)
}
export async function looped(user, types) {
  for (const type of types) {
    const q = startFactory()
    appendTypeFilter(q, user, type)
    await read(q)
  }
}
