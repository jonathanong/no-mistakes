import type sql from "@example/db";
async function build(id: string) { return sql`SELECT ${id}`; }
await build("1");
