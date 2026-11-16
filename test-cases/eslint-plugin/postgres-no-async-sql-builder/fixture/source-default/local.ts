function sql(text: TemplateStringsArray) { return text[0]; }
async function build(id: string) { return sql`SELECT ${id}`; }
await build("1");
