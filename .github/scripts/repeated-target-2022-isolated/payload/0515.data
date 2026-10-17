import { read } from '@example/db'

function run(sql: string) {
  return read(sql)
}

export const callbacks = [...[run]]
run('/* exported/spread */ SELECT id FROM posts')
