declare const flag: string;
const db = require(flag);

function run(query: (sql: string) => Promise<unknown>) {
  return query;
}

run(db.read);
run(flag ? db.read : undefined);
let { read } = require(flag);
run(read);
