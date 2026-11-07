import { beginTransaction, type Query } from "db"
import { providers, type Provider, regions, type Region } from "@scope/pkg/config"

export async function load(provider: Provider) {
  if (!providers.includes(provider)) throw new Error("unknown provider")
  const table = `${provider}_accounts`
  provider = process.env.PROVIDER as Provider
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
