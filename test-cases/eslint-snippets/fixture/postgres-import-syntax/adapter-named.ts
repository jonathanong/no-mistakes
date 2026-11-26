declare const flag: string;

const { read } = require(flag);

function run(query: (sql: string) => Promise<unknown>) {
  return query;
}

run(read);
