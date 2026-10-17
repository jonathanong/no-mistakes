import { read } from '@example/db'

function run(sql: string) {
  return read(sql)
}

const handlers = { run }
export const api = { ...handlers }

run('/* exported/spread */ SELECT id FROM posts')
