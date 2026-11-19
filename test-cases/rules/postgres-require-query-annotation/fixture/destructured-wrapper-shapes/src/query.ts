import { read, type TransactionQuery } from '@example/psql'
import sql from 'sql-template-strings'
import { ITEM_COLUMNS, itemPage } from './builders'
import { ITEMS_TABLE, PAGE } from './consts'
import { runInTx } from './tx'

export async function renameItem<T>(label: string, itemId: string) {
  return runInTx(undefined, async (transaction: TransactionQuery) => {
    const { rows } = await transaction<T>(
      sql`/* rename */
    UPDATE items
    SET label = ${label}
    WHERE id = ${itemId}
    RETURNING `.append(ITEM_COLUMNS),
    )
    return rows[0]
  })
}

export async function undefinedBuilderArgument() {
  const { rows } = await read<{ id: string }>(
    itemPage(undefined, PAGE),
  )
  return rows
}

export async function pagedBuilderArgument(after: { id: string }, pageSize: number) {
  const { rows } = await read<{ id: string }>(itemPage(after, pageSize))
  return rows
}

export async function renamedImportedConstTemplate() {
  const { rows: countRows } = await read(
    `/* present */ SELECT 1 AS present FROM ${ITEMS_TABLE} LIMIT 1`,
    [],
  )
  if (countRows.length === 0) return
}
