import { read } from '@example/db'

function run(sql: string) {
  return read(sql)
}

export const api = { run }
run('/* exported/record */ SELECT id FROM posts')
