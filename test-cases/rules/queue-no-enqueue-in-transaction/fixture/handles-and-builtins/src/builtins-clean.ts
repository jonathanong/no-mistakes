import { withTx } from './tx'
import defaultUtils, { fetchLabels, formatId } from '@sample/utils'

function extractId(row: { id: string }): string {
  return row.id.trim().toLowerCase()
}

function getLocalRows(): { id: string }[] {
  return []
}

export class ItemRunner {
  items: string[] = []

  run(rows: { id: string; tags?: string[] }[], sep: string, extra?: { id: string }[]) {
    return withTx(async query => {
      const seen = new Set<string>()
      const byId = new Map<string, string>()
      const ids = (extra ?? undefined ?? rows)
        .filter(Boolean)
        .map(extractId)
        .map(formatId)
        .map(defaultUtils)
        .map(defaultUtils.format)
        .filter(id => id.length > 0)
      const localSep = '-'
      const trimmedSep = localSep.trim()
      const seenAlias = seen
      for (const id of ids) {
        seen.add(id)
        byId.set(id, id.replace(/\s+/g, sep).replaceAll('_', localSep).replace(/x/, `a-${sep}`))
      }
      const repeatIds = rows.map(extractId)
      const typedQueryRef = query<{ id: string }>
      const instantiatedUtil = (defaultUtils<{ id: string }>)('x')
      const parsedNum = parseInt('10', 10)
      const localIds = getLocalRows().map(r => r.id)
      const fetchedLabels = fetchLabels().map((label: string) => label.trim())
      const keys = Object.keys(Object.fromEntries(byId.entries()))
      const joined = Array.from(seen).join(',')
      const payload = JSON.stringify({ keys, joined, ok: Array.isArray(ids) }, null, 2)
      const filteredPayload = JSON.stringify({ keys }, ['keys'])
      const parsed = JSON.parse(payload)
      const count = Number.isFinite(ids.length) ? Math.min(10, ids.length) : 0
      const absValues = [1, -2].map(Math.abs)
      const fixedPi = Math.PI.toFixed(2)
      const trimmed = `prefix-${rows[0]?.id ?? joined}`.trim()
      const raw = String.raw`SELECT ${String.fromCharCode(49)}`
      const stamp = new Date().toISOString()
      const ownItems = this.items.map(item => item.trim())
      const awaitedIds = (await query<{ id: string }>(`SELECT id FROM items`)).map(r => r.id)
      const nested = await Promise.all(
        awaitedIds.map(id => query<{ id: string }>`SELECT ${id}`),
      )
      await Promise.all(awaitedIds.map(query))
      const tried = await Promise.try(() => count)
      return {
        count,
        parsedNum,
        repeatIds,
        typedQueryRef,
        instantiatedUtil,
        parsed,
        filteredPayload,
        raw,
        stamp,
        nested,
        absValues,
        fixedPi,
        trimmed,
        trimmedSep,
        ownItems,
        localIds,
        fetchedLabels,
        tried,
        hasFirst: seen.has(ids[0] ?? ''),
        hasAlias: seenAlias.has(ids[0] ?? ''),
      }
    })
  }
}
