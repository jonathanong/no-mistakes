declare function unknown(value: unknown): void

// Only the same fresh caller builder reaches execute; append keeps its prefix.
export async function page(base: any, execute: (value: any) => Promise<unknown>) {
  base.append(' WHERE active = true')
  return execute(base)
}

export async function views(analyze: (value: string) => Promise<unknown>) {
  for (const view of ['one', 'two'] as const) {
    await analyze(`/* views */ ANALYZE ${view}`)
  }
}

export async function mixed(base: any, execute: (value: any) => Promise<unknown>) {
  await execute(base)
  await execute('SELECT 1')
}

export async function escaped(base: any, execute: (value: any) => Promise<unknown>) {
  unknown(execute)
  await execute(base)
}

export async function aliased(base: any, execute: (value: any) => Promise<unknown>) {
  const alias = base.append(' WHERE active = true')
  unknown(alias)
  await execute(base)
}

export async function replaced(base: any, execute: (value: any) => Promise<unknown>) {
  base = 'SELECT 1'
  await execute(base)
}

export async function prototype(base: any, execute: (value: any) => Promise<unknown>) {
  Object.setPrototypeOf(base, { append: () => 'SELECT 1' })
  await execute(base)
}
