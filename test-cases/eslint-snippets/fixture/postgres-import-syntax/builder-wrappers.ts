declare function sql(text: TemplateStringsArray): string;
type Row = { id: number };

async function wrappedCall() {
  return sql`SELECT 1`;
}
await wrappedCall!();

const wrappedDef = (async () => sql`SELECT 1`) satisfies () => Promise<unknown>;

declare const db: { sql: (text: TemplateStringsArray) => string };

async function memberTag() {
  return (db.sql<Row>)`SELECT 1`;
}
