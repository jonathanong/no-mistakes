import sql from 'sql-template-strings'
export function appendForeign(q, user) {
  if (user?.admin) { q.append(sql`TRUE`); return }
  q.append(sql`visible = true`)
}
export function appendOther(q, value) { q.append(value) }
export function appendDefaulted(q, { suffix = 'x' } = {}) { q.append(suffix) }
export function appendCaptured(q) { schedule(() => { q.append('x') }) }
export function appendReturning(q) { return q.append('x') }
export function appendBroken(q) { q.text = 'SELECT 2' }
export function appendOptional(q) { q.append('x') }
export function appendSpread(q) { q.append('x') }
export default function appendDefault(q) { q.append('x') }
