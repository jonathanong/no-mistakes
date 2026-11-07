import { beginTransaction, type Query } from "db"
import { providers, type Provider, regions, type Region } from "@scope/pkg/config"

export async function load(provider: Provider) {
  if (!providers.includes(provider)) throw new Error("unknown provider")
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${provider}`)
  return query(`/* second */ SELECT 1 FROM ${provider}`)
}
