import sql from 'sql-template-strings'

declare const runtimeFragments: unknown[]

export function build(): void {
  const query = sql`AND true`
  query.append(...runtimeFragments)
}
