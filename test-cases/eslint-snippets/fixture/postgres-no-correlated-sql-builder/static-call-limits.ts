import sql from "sql-template-strings";

type Builder = { append(text: unknown): unknown };

const name = "append";

export function computed(query: Builder, access: string) {
  if (access === "owned") query[name](sql`AND account_id = $1`);
  if (access === "member") query[name](sql`AND member_id = $1`);
}

export function otherMethod(query: Builder, access: string) {
  if (access === "owned") query.other(sql`AND account_id = $1`);
  if (access === "member") query.other(sql`AND member_id = $1`);
}

export function nestedObject(box: { query: Builder }, access: string) {
  if (access === "owned") box.query.append(sql`AND account_id = $1`);
  if (access === "member") box.query.append(sql`AND member_id = $1`);
}

export function spreadCall(query: Builder, access: string) {
  const piece = sql`AND account_id = $1`;
  if (access === "owned") query.append.call(query, ...[piece]);
  if (access === "member") query.append.call(query, ...[piece]);
}

export function shortCall(query: Builder, access: string) {
  if (access === "owned") query.append.call(sql`AND account_id = $1`);
  if (access === "member") query.append.call(sql`AND member_id = $1`);
}

export function letForwarder(query: Builder, access: string) {
  let add = (text: unknown) => query.append(text);
  if (access === "owned") add(sql`AND account_id = $1`);
  if (access === "member") add(sql`AND member_id = $1`);
}

export function memberAlias(query: Builder, access: string) {
  const add = query.append;
  if (access === "owned") add(sql`AND account_id = $1`);
  if (access === "member") add(sql`AND member_id = $1`);
}

export function twoParams(query: Builder, access: string) {
  const add = (text: unknown, extra?: unknown) => query.append(text ?? extra);
  if (access === "owned") add(sql`AND account_id = $1`);
  if (access === "member") add(sql`AND member_id = $1`);
}

export function otherCall(query: Builder, access: string) {
  if (access === "owned") query.other.call(query, sql`AND account_id = $1`);
  if (access === "member") query.other.call(query, sql`AND member_id = $1`);
}

export function computedCall(query: Builder, access: string) {
  if (access === "owned") query[name].call(query, sql`AND account_id = $1`);
  if (access === "member") query[name].call(query, sql`AND member_id = $1`);
}

export function memberCall(box: { query: Builder }, access: string) {
  if (access === "owned") box.query.append.call(box.query, sql`AND account_id = $1`);
  if (access === "member") box.query.append.call(box.query, sql`AND member_id = $1`);
}

export function unbound(query: Builder, access: string) {
  if (access === "owned") missing(sql`AND account_id = $1`);
  if (access === "member") missing(sql`AND member_id = $1`);
}

export function redeclared(query: Builder, access: string) {
  // Two `var` bindings are not one const forwarder.
  var add = (text: unknown) => query.append(text);
  var add = (text: unknown) => query.append(text);
  if (access === "owned") add(sql`AND account_id = $1`);
  if (access === "member") add(sql`AND member_id = $1`);
}

export function tooMany(query: Builder, access: string) {
  const add = (text: unknown) => query.append(text);
  if (access === "owned") add(sql`AND account_id = $1`, sql`AND extra`);
  if (access === "member") add(sql`AND member_id = $1`, sql`AND extra`);
}

export function spreadForward(query: Builder, access: string) {
  const add = (text: unknown) => query.append(text);
  const piece = sql`AND account_id = $1`;
  if (access === "owned") add(...[piece]);
  if (access === "member") add(...[piece]);
}

export function notSql(query: Builder, access: string) {
  const add = (text: unknown) => query.append(text);
  if (access === "owned") add(query);
  if (access === "member") add(query);
}

export function patternParam(query: Builder, access: string) {
  // The parameter is a pattern, so the call is not a single forwarded binding.
  const add = ({ text }: { text: unknown }) => query.append(text);
  if (access === "owned") add(sql`AND account_id = $1`);
  if (access === "member") add(sql`AND member_id = $1`);
}

export function extraStatement(query: Builder, access: string) {
  const add = (text: unknown) => {
    query.append(text);
    return text;
  };
  if (access === "owned") add(sql`AND account_id = $1`);
  if (access === "member") add(sql`AND member_id = $1`);
}
