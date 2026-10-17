import { read } from '@example/db'

function run(sql: string) {
  return read(sql)
}

export const callback = [run][0]
run('/* exported/indexed */ SELECT id FROM posts')
