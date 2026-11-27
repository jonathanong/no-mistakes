declare const flag: string;
declare const items: string[];

const { read } = require(flag);
await Promise.all(items.map(read));

const db = require(flag);
const wrapped = (id: string) => db.read(id);
await Promise.all(items.map(wrapped));
