import sql from 'sql-template-strings'

export function report(q, { includeAvailable = false } = {}) {
  q.append(sql` WHERE active = true`)
  if (includeAvailable) q.append(sql` AND available = true`)
}
export function topics(q, ids, postAlias = 'p') {
  q.append(sql` WHERE topic_id = ANY(${ids})`)
  q.append(postAlias === 'posts' ? sql` AND posts.active` : sql` AND p.active`)
}
export function plainDestructure(q, { suffix }) { q.append(suffix) }
// Plain destructures beyond the new bounded mask retain their earlier proof.
export function widePlain(q, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, { suffix }) { q.append(suffix) }
export function badFirst(q = opaque(), options = {}) { q.append(' ') }
export function badIdentifier(q, flag = opaque(q)) { q.append(flag) }
export function badProperty(q, { flag = opaque(q) } = {}) { q.append(flag) }
export function badOuter(q, { flag } = { get flag() { return true } }) { q.append(flag) }
export function badComputed(q, { [key]: flag } = {}) { q.append(flag) }
export function badProto(q, { __proto__: flag } = {}) { q.append(flag) }
