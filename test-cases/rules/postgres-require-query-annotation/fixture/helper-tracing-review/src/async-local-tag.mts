async function sql(strings: TemplateStringsArray, ...values: unknown[]) { return strings.join('?'); }
export function asyncLocalStatement() { return sql`/* async local tag */ SELECT 1`; }
