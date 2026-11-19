import { write } from '@example/psql'

export function run(text: string) {
  return write(text)
}

run('/* a */ select 1')

export function external(dynamic: string) {
  return run('select ' + dynamic)
}
