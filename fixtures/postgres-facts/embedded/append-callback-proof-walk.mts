type Sql = { append(value: unknown): void };
type Query = (statement: Sql) => Promise<{ rows: string[] }>;

export async function listWidgets(statement: Sql, ready: boolean, query: Query) {
  if (ready) statement.append(sql` and active`);
  return query(statement);
}

export async function listItems(statement: Sql, ready: boolean, query: Query) {
  if (ready) statement.append(sql` and active`);
  const rows = await query(statement);
  return rows.map((item) => item.id);
}

export async function listOrders(statement: Sql, ready: boolean) {
  if (ready) statement.append(sql` and active`);
  while (ready) {
    break;
  }
  return statement;
}

export async function listShipments(statement: Sql, ready: boolean) {
  if (ready) statement.append(sql` and active`);
  return statement;
}

export async function filterWidgets(selectWhere: Sql, f: boolean, query: Query) {
  if (f) selectWhere.append(sql` AND a`);
  const { rows } = await query(selectWhere);
  return rows;
}
