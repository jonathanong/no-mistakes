import { read } from '@example/db'

function run(sql: string) {
  return read(sql)
}

export const callback = (0, run)
run('/* exported/sequence */ SELECT id FROM posts')
