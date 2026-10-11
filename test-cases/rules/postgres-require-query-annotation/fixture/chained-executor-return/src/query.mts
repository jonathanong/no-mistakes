import sql from 'sql-template-strings'
import { write } from '@example/db'

write('/* direct */ SELECT 1')
write('/* caught */ SELECT 1').catch(() => {})
write('/* continued */ SELECT 1').then(() => {})
write('/* finalized */ SELECT 1').finally(() => {})
write(sql`/* tagged */ SELECT id FROM items WHERE id = ${'id'}`).catch(() => {})
const result = write('/* assigned */ SELECT 1')
result.catch(() => {})
write('/* computed */ SELECT 1')['catch'](() => {})
write('/* optional */ SELECT 1')?.catch(() => {})
write('/* optional method */ SELECT 1').catch?.(() => {})
write('/* repeated */ SELECT 1').then(() => {}).catch(() => {})

write('SELECT 1').catch(() => {}) // finding: missing
write(runtimeSql).catch(() => {}) // finding: opaque

// The callback's own executor is a distinct call, even under an unknown return method.
write('/* parent */ SELECT 1').catch(() => write('SELECT 2')) // finding: missing
write('/* parent */ SELECT 1').finally(() => write(runtimeSql)) // finding: opaque

const before = sql`/* before */ SELECT 1`
opaqueMutate(before)
write(before).catch(() => {}) // finding: opaque

// A callback runs after the inner write and cannot rewrite its recorded SQL.
const after = sql`/* after */ SELECT 1`
write(after).catch(() => opaqueMutate(after))
