import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";
import { assertSafeAlias, deleted, access } from "./clauses.mts";

export type OuterOptions = { strict?: boolean; includeDrafts?: boolean };

// A helper that calls a private helper, which calls helpers imported from
// another module. Literal aliases flow down and are spliced into raw appends.
export function buildOuter(a: string, b: string, options?: OuterOptions): SQLStatement {
  return buildInner(a, b, options, true);
}

function buildInner(
  a: string,
  b: string,
  options: OuterOptions | undefined,
  requireLive = true,
): SQLStatement {
  assertSafeAlias(a);
  assertSafeAlias(b);

  const statement = sql`(`
    .append(deleted(a, b))
    .append(sql` AND `)
    .append(`${a}.approved_at IS NOT NULL`);
  if (requireLive) {
    statement.append(sql` AND `).append(`${b}.live_at IS NOT NULL`);
  }
  // A ternary append of two literal-spliced strings is two alternatives, but
  // `options` is omitted by the caller, so only the `else` arm is possible.
  statement.append(sql` AND `).append(
    options?.strict === true ? `${b}.audience IN ('everyone', 'users')` : `${b}.audience = 'everyone'`,
  );
  const gate = access(b, options?.includeDrafts === true);
  return statement.append(sql` AND `).append(gate).append(sql`)`);
}
