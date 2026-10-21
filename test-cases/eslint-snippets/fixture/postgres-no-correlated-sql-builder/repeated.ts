import sql from "sql-template-strings";

type Builder = { append(text: unknown): unknown };

export function byAccess(query: Builder, access: string) {
  if (access === "owned") query.append(sql`AND account_id = $1`);
  if (access === "member") {
    query.append(sql`AND member_id = $1`);
  }
  if ("all" == access) query.append(`AND true`);
  if (access === "literal") query.append("AND literal");
}

export const expression = function expression(query: Builder, access: string) {
  if (access === "owned") query.append(sql`AND account_id = $1`);
  if (access === "member") query.append(sql`AND member_id = $1`);
};

export function chained(query: Builder, access: string) {
  if (access === "owned") query.append(sql`AND account_id = $1`);
  else if (access === "member") query.append(sql`AND member_id = $1`);
}

export function inner(query: Builder, access: string) {
  const nested = (flag: string) => {
    if (flag === "owned") query.append(sql`AND account_id = $1`);
    if (flag === "member") query.append(sql`AND member_id = $1`);
  };
  if (access === "owned") query.append(sql`AND id = $1`);
  return nested;
}
