import { read } from '@example/db'

function run(sql: string) {
  return read(sql)
}

function noop(_sql: string) {}

export function api(flag: boolean) {
  return flag ? run : noop
}

run('/* exported/conditional */ SELECT id FROM posts')
