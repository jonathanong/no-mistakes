import { read, write } from '@example/psql'
import { itemPage } from './builders'
import { captureAll } from './capture'
import { ITEMS_TABLE, PAGE } from './consts'

async function insertItem(): Promise<string> {
  const { rows } = await write<{ id: string }>(
    `/* insertItem */ INSERT INTO items (id, created_at) VALUES (gen_random_uuid(), now()) RETURNING id`,
  )
  return rows[0].id
}

export async function runJobs(): Promise<void> {
  const knownId = await insertItem()
  await captureAll(
    'page',
    async () => {
      const { rows } = await read<{ id: string }>(itemPage(undefined, PAGE))
      if (!rows.some(row => row.id === knownId)) throw new Error('missing')
    },
    undefined,
  )
  await captureAll(
    'present',
    async () => {
      const { rows: countRows } = await read(
        `/* present */ SELECT 1 AS present FROM ${ITEMS_TABLE} LIMIT 1`,
        [],
      )
      if (countRows.length !== 1) throw new Error('empty')
    },
    undefined,
  )
}
