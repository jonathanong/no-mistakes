import { read } from '@example/db'

function run(sql: string) {
  return read(sql)
}

var first = second
var second = first
export const api = { first, run }
run('/* exported/cyclic */ SELECT id FROM posts')
