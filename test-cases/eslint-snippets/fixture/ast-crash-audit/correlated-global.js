/* global accessKind */
export function build(query) {
  if (accessKind === "orders") query.append("SELECT id FROM orders");
  if (accessKind === "accounts") query.append("SELECT id FROM accounts");
  return query;
}
