import sql from "sql-template-strings";

type Builder = { append(text: unknown): unknown };

const db = { sql };

export function brackets(query: Builder, access: string) {
  if (access === "owned") query["append"](sql`AND account_id = $1`);
  if (access === "member") query["append"](sql`AND member_id = $1`);
}

export function templateKey(query: Builder, access: string) {
  if (access === "owned") query[`append`](sql`AND account_id = $1`);
  if (access === "member") query[`append`](sql`AND member_id = $1`);
}

export function memberTag(query: Builder, access: string) {
  if (access === "owned") query.append(db.sql`AND account_id = $1`);
  if (access === "member") query.append(db.sql`AND member_id = $1`);
}

export function literalTag(query: Builder, access: string) {
  if (access === "owned") query.append(db["sql"]`AND account_id = $1`);
  if (access === "member") query.append(db["sql"]`AND member_id = $1`);
}

export function templateTag(query: Builder, access: string) {
  if (access === "owned") query.append(db[`sql`]`AND account_id = $1`);
  if (access === "member") query.append(db[`sql`]`AND member_id = $1`);
}

export function otherReceiver(other: Builder, query: Builder, access: string) {
  if (access === "owned") query.append.call(other, sql`AND account_id = $1`);
  if (access === "member") query.append.call(other, sql`AND member_id = $1`);
}

export function forwardedCall(query: Builder, access: string) {
  if (access === "owned") query.append.call(query, sql`AND account_id = $1`);
  if (access === "member") query.append.call(query, sql`AND member_id = $1`);
}

export function arrowed(query: Builder, access: string) {
  const add = (text: unknown) => query.append(text);
  if (access === "owned") add(sql`AND account_id = $1`);
  if (access === "member") add(sql`AND member_id = $1`);
}

export function declared(query: Builder, access: string) {
  function add(text: unknown) {
    return query.append(text);
  }
  if (access === "owned") add(sql`AND account_id = $1`);
  if (access === "member") add(sql`AND member_id = $1`);
}
