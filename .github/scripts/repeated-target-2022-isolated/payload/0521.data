import { read } from '@example/db'

function run(sql: string) {
  return read(sql)
}

const key = Math.random() ? 'run' : 'other'
export const callback = { run }[key]
run('/* exported/dynamic-key */ SELECT id FROM posts')
