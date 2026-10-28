import sql from 'sql-template-strings'
import { read } from '@bench/db'
import {
  named, differentNames, missing, mixed, unknown, aliased, mutation, capture,
  hiddenReturn, nestedBranch, throwsBeforeReturn, independentNestedReturn,
  differentOwner, fallthrough, untrusted, switched,
} from './helpers.mts'
declare const flag: boolean, options: { owner: string; cursor?: string; limit: number }, opaque: unknown

const query = named(flag, options)
read(query) // proof: Literal
read(differentNames(flag)) // proof: Literal
read(missing(flag)) // proof: Missing
read(mixed(flag)) // proof: Unproven
read(unknown(flag, opaque)) // proof: Unproven
read(aliased(flag)) // proof: Unproven
read(mutation(flag)) // proof: Unproven
read(capture(flag)) // proof: Unproven
read(hiddenReturn(flag, flag)) // proof: Unproven
read(nestedBranch(flag, flag)) // proof: Unproven
read(throwsBeforeReturn(flag, flag)) // proof: Literal
read(independentNestedReturn(flag)) // proof: Literal
read(differentOwner(flag, opaque)) // proof: Unproven
read(fallthrough(flag)) // proof: Unproven
read(untrusted(flag)) // proof: Unproven
read(switched('a')) // proof: Literal

function local(flag) {
  if (flag) {
    const query = sql`/* local/early */ SELECT 1`
    query.append(' LIMIT 1')
    return query
  }
  const query = sql`/* local/final */ SELECT 2`
  return query
}
function localMissing(flag) {
  if (flag) { log(flag); return sql`SELECT 1` }
  return sql`SELECT 2`
}
read(local(flag)) // proof: Literal
read(localMissing(flag)) // proof: Missing
