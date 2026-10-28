import sql from 'sql-template-strings'
import other from 'custom-sql'
export const exportedData = 1
export function good(query, value) { query.append(sql` ${value}`) }
function noParameters() { query.append(sql` suffix`) }
function notCall(query) { query }
function ordinarySuffix(query) { query.append('suffix') }
function optional(query) { query.append?.(sql` suffix`) }
function wrongCallee(query) { consume(sql` suffix`) }
function wrongReceiver(query) { holder.query.append(sql` suffix`) }
function computed(query) { query['append'](sql` suffix`) }
function wrongTag(query) { query.append(other` suffix`) }
function typeUse(query, value: typeof query) { query.append(sql` suffix`) }
function changed(query) { query.append(sql` suffix`) }
changed = opaque
function duplicate(query) { query.append(sql` suffix`) }
function duplicate(query) { mutate(query) }
function run(value) {
  const single = sql`/* single */ SELECT 1`
  single.append(sql` suffix`)
  good(single, value)
  consume(single)
  const none = sql`/* none */ SELECT 1`
  none.append(sql` suffix`)
  const many = sql`/* many */ SELECT 1`
  consume(many)
  consume(many)
  const captured = sql`/* captured */ SELECT 1`
  register(() => consume(captured))
  const written = sql`/* written */ SELECT 1`
  written = replacement
}
