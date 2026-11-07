import { beginTransaction, type Query } from "db"
import { providers, type Provider, regions, type Region } from "@scope/pkg/config"

export async function load(input: { provider: Provider }) {
  if (!providers.includes(input.provider)) throw new Error("unknown provider")
  const table = `${input.provider}_accounts`
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
