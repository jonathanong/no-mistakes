import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { clobber } from './j-clobber-sql.mts'
import { unknownPage } from './i-unknown-page.mts'
export async function mapped(winners) {
  const query = sql`/* mapped */ SELECT 1 FROM t`
  winners.map(filter => query.append(filter))
  await read(query)
}
export async function chainedMethod(winners) {
  const query = sql`/* chainedMethod */ SELECT 1 FROM t`
  winners.forEach(filter => query.append(sql` AND `).setName(filter))
  await read(query)
}
export async function clobbered(winners) {
  const query = sql`/* clobbered */ SELECT 1 FROM t`
  clobber(query)
  winners.forEach(filter => query.append(filter))
  await read(query)
}
export async function unknownCaptured(winners) {
  const query = sql`/* unknownCaptured */ SELECT 1 FROM t`
  unknownPage(query)
  winners.forEach(filter => { query.append(filter) })
  await read(query)
}
export async function optionalForEach(winners) {
  const query = sql`/* optionalForEach */ SELECT 1 FROM t`
  winners?.forEach(filter => query.append(filter))
  await read(query)
}
