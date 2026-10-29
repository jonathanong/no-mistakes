import { read } from '@example/db'

function table(family: string): string {
  switch (family) {
    case 'one': return 'orders'
    case 'two': return 'accounts'
    default: throw new Error('unsupported table')
  }
}

// A no-hole ordinary template is an inert selector argument.
export function staticTemplate() {
  return read(`SELECT id FROM ${table(`one`)} WHERE id = $1`, [1])
}

// A spread can run arbitrary iteration before the SQL read.
export function spreadArgument(family: string) {
  return read(`SELECT id FROM ${table(...[family])} WHERE id = $1`, [1])
}
