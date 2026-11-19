import { write } from '@example/psql'

function run(text: string) {
  return write(text)
}

export function annotated() {
  return run('/* a */ select 1')
}

export function other(dynamic: string, rows: string[]) {
  for (const row of rows) {
    rows.length
  }
  return run(dynamic)
}
