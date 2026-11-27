declare const flag: string;
declare const items: string[];
const db = require(flag);

await Promise.all(items.map((id) => db.read("SELECT 1", [id])));
