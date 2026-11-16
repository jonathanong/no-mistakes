import { default as tag } from "@example/db";
async function build(id: string) { return tag`SELECT ${id}`; }
await build("1");
