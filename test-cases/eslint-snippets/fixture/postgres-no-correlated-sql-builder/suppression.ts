import sql from "sql-template-strings";

type Builder = { append(text: unknown): unknown };

export function kept(query: Builder, access: string) {
  // no-mistakes-disable-next-line postgres-no-correlated-sql-builder -- reviewed builder
  if (access === "owned") query.append(sql`AND account_id = $1`);
  if (access === "member") query.append(sql`AND member_id = $1`);
}

export function reported(query: Builder, access: string) {
  if (access === "owned") query.append(sql`AND account_id = $1`);
  if (access === "member") query.append(sql`AND member_id = $1`);
}
