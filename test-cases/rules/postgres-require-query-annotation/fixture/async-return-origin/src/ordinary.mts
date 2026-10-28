import { read } from '@bench/db'
export async function ordinary(value) {
  const result = await unrelated(value)
  read(result)
}
