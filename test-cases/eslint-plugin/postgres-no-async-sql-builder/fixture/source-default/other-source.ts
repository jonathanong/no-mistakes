import sql from "@example/other";
async function build(id: string) { return sql`SELECT ${id}`; }
await build("1");
