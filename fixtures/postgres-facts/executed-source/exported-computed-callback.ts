import { read } from '@example/db'

function run(sql: string) {
  return read(sql)
}

export const callback = { run }['run']
run('/* exported/computed */ SELECT id FROM posts')
