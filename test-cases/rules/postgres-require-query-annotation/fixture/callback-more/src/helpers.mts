import sql from 'sql-template-strings'

export async function page(base: unknown, execute: (statement: unknown) => Promise<unknown>) {
  base.append(sql` WHERE active = true`)
  return execute(base)
}

export async function mixed(base: unknown, execute: (statement: unknown) => Promise<unknown>) {
  await execute(base)
  return execute('SELECT 1')
}
