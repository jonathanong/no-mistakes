import { write } from '@example/psql'
import sql from 'sql-template-strings'

function clobber(statement: { text: string }) {
  statement.text = 'delete from widgets'
  return true
}

export async function guardTestMutates() {
  const statement = sql`/* a */ select 1`
  if (clobber(statement)) return undefined
  return write(statement)
}

export async function guardArmMutates(flag: boolean) {
  const statement = sql`/* b */ select 1`
  if (flag) {
    clobber(statement)
    return undefined
  }
  return write(statement)
}
