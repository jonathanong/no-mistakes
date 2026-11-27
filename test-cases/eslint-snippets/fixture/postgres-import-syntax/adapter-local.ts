function read(sql: string) {
  return sql;
}

function run(query: (sql: string) => Promise<unknown>) {
  return query;
}

run(read);
