import { beginTransaction, type Query } from "db"
import { providers, type Provider, regions, type Region } from "@scope/pkg/config"

export async function load(provider: Provider) {
  if (!providers.includes(provider)) throw new Error("unknown provider")
  const table = `${provider}_accounts`
  const captured = table
  const select = async (query: Query) => {
    const first = await query(`/* first */ SELECT 1 FROM ${captured}`)
    const second = await query(`/* second */ SELECT 1 FROM ${captured}`)
    return [first, second]
  }
  await using query = await beginTransaction()
  return select(query)
}
