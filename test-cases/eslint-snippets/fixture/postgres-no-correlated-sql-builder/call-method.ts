import sql from "sql-template-strings";

type Builder = { call(text: unknown): unknown };

// A method whose name is `call` is the builder method, not Function.prototype forwarding.
export function namedCall(query: Builder, access: string) {
  if (access === "owned") query.call(sql`AND account_id = $1`);
  if (access === "member") query.call(sql`AND member_id = $1`);
}
