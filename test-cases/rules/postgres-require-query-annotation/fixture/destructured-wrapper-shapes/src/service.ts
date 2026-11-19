import { read } from '@example/psql'
import { itemPage, type Cursor } from './builders'

async function isEnabled(): Promise<boolean> {
  return true
}

export async function listItems(data: { after?: Cursor }, pageSize: number) {
  if (!(await isEnabled())) return { hasMore: false, total: 0 }
  const { after } = data
  const { rows } = await read<{ id: string; created_at: Date }>(itemPage(after, pageSize))
  const last = rows.at(-1)
  return { hasMore: true, total: rows.length, last }
}
