import { beginTransaction } from "db"

const providers = ["facebook", "github"] as const
type Provider = (typeof providers)[number]
export async function load(provider: Provider, flag: boolean) {
  if (!providers.includes(provider)) throw new Error("unknown provider")
  const table = `${provider}_accounts`
  await using query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  if (flag) {
    const first = await query(`/* branch first */ SELECT 1 FROM ${table}`)
    const [second] = await query(`/* branch second */ SELECT 1 FROM ${table}`)
    return [first, second]
  }
  const first = await query(`/* other first */ SELECT 1 FROM ${table}`)
  return query(`/* other second */ SELECT 1 FROM ${table}`)
}
