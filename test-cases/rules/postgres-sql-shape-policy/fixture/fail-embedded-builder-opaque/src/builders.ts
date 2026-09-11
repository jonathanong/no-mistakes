import sql from 'sql-template-strings'

declare const runtimeFragments: unknown[]

export function build(): void {
  sql`AND true`.append(...runtimeFragments)
}
