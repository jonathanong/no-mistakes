import sql from "sql-template-strings";
import { read } from "@example/db";

// Each IIFE below leaves the declared `const` opaque, so each read stays
// unanalyzable.

export async function withParameter(id: string) {
  const built = ((table: string) => {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${id}`;
    return query;
  })("p");
  return read(built);
}

export async function withArgument(id: string) {
  const built = (function () {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${id}`;
    return query;
  })(id);
  return read(built);
}

export async function asynchronous(id: string) {
  const built = await (async () => {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${id}`;
    return query;
  })();
  return read(built);
}

export async function generator(id: string) {
  const built = (function* () {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${id}`;
    return query;
  })();
  return read(built);
}

export async function usingThis(id: string) {
  const built = (function () {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${this.id}`;
    return query;
  })();
  return read(built);
}

export async function usingArguments(id: string) {
  const built = (function () {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${arguments.length}`;
    return query;
  })();
  return read(built);
}

export async function multipleReturns(id: string, flag: boolean) {
  const built = (() => {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${id}`;
    if (flag) return query;
    query.append(sql` AND p.tier = 'a'`);
    return query;
  })();
  return read(built);
}

export async function returnsOuterBuilder(id: string) {
  const outer = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${id}`;
  const built = (() => {
    outer.append(sql` AND p.tier = 'a'`);
    return outer;
  })();
  return read(built);
}

export async function returnsAnExpression(id: string) {
  const built = (() => {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${id}`;
    return query.append(sql` AND p.tier = 'a'`);
  })();
  return read(built);
}

export async function statementsAfterTheReturn(id: string) {
  const built = (() => {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${id}`;
    return query;
    query.append(sql` AND p.tier = 'a'`);
  })();
  return read(built);
}

export async function namedFunctionExpression(id: string) {
  const built = (function build() {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${id}`;
    return query;
  })();
  return read(built);
}

export async function reassignable(id: string) {
  let built = (() => {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${id}`;
    return query;
  })();
  return read(built);
}
