import { read } from '@example/db'

export const api = {
  run: (sql: string) => read(sql),
}

api.run('/* exported/inline */ SELECT 1')
