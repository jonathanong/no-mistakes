import sql from "sql-template-strings";
import html from "html-template";

type Builder = { append(text: unknown): unknown; push(text: unknown): unknown };

export function shapes(query: Builder, access: string) {
  if (access === `owned`) query.append(sql`AND account_id = $1`);
  if (access === `own${"ed"}`) query.append(sql`AND member_id = $1`);
  if (access === 1) query.append(1);
}

export function once(query: Builder, access: string) {
  if (access === "owned") query.append(sql`AND account_id = $1`);
}

export function different(query: Builder, access: string, other: string) {
  if (access === "owned") query.append(sql`AND account_id = $1`);
  if (other === "member") query.append(sql`AND member_id = $1`);
}

export function builders(query: Builder, limit: Builder, access: string) {
  if (access === "owned") query.append(sql`AND account_id = $1`);
  if (access === "member") limit.append(sql`AND member_id = $1`);
}

export function notSql(query: Builder, access: string, other: string) {
  if (access === other) query.append(sql`AND account_id = $1`);
  if (access) query.append(sql`AND member_id = $1`);
  if (access === "owned") query.append(access);
  if (access === "member") query.append(access, 1);
  if (access === "all") query.append(html`AND true`);
  if (access === "legacy") query["append"](sql`AND legacy = $1`);
  if (access === "extra") query.push(sql`AND extra = $1`);
}

export function member(box: { query: Builder }, access: string) {
  if (access === "owned") box.query.append(sql`AND account_id = $1`);
  if (access === "member") box.query.append(sql`AND member_id = $1`);
}

declare function append(text: unknown): unknown;

export function bare(access: string) {
  if (access === "owned") append(sql`AND account_id = $1`);
  if (access === "member") append(sql`AND member_id = $1`);
}

export function lostBuilder(access: string) {
  if (access === "owned") missingBuilder.append(sql`AND account_id = $1`);
  if (access === "member") missingBuilder.append(sql`AND member_id = $1`);
}

export function unresolved(query: Builder) {
  if (missing === "owned") query.append(sql`AND account_id = $1`);
  if (missing === "member") query.append(sql`AND member_id = $1`);
}

export function shadow(query: Builder, access: string) {
  if (access === "owned") query.append(sql`AND account_id = $1`);
  if (access === "member") {
    const query = { append(_text: unknown) {} };
    query.append(sql`AND member_id = $1`);
  }
}

