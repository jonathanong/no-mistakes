import { type Query } from 'db'
import { getIdentifiers } from './providers.ts'

export async function getAccount(provider: 'github' | 'gitlab', query: Query) {
  const config = getIdentifiers(provider)
  return query(`/* getAccount */ SELECT
    ${config.accountIdColumn === 'id' ? 'id AS account_id,' : ''}
    ${config.providerUserIdColumn} AS provider_user_id,
    ${config.emailColumn} AS email`)
}
