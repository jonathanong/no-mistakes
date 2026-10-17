import { read } from '@example/db'

function run(sql: string) {
  return read(sql)
}

export const callback = run && run
run('/* exported/logical */ SELECT id FROM posts')
