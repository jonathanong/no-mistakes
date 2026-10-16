import { read } from '@example/db'

export const query = String.raw`/* raw/read */ SELECT id FROM posts`

export function list() {
  return read(query)
}
