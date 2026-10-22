import * as db from "@example/db";
import { helper } from "@example/db";
import * as other from "@example/other";

const name = "sql";
const holder = { sql: 1 };
// A computed identifier, a default, a member alias, and a non-const binding are not `db.sql`.
const { [name]: computed } = db;
const { sql: withDefault = helper } = db;
const tag = db.sql;
let { sql: fromLet } = db;
// Two `var` bindings are not one const destructure.
var doubled = db.sql;
var doubled = db.sql;
const { sql: fromMember } = db.helper;
const { sql: fromNamed } = helper;
const { sql: fromOther } = other;
const { query } = db;
const { sql: fromHolder } = holder;

async function missingTag() {
  return missing`SELECT 1`;
}
async function parameterTag(local: string) {
  return local`SELECT 1`;
}
async function computedMember() {
  return db[name]`SELECT 1`;
}
async function templateExpr() {
  return db[`${name}`]`SELECT 1`;
}
async function computedDestructure() {
  return computed`SELECT 1`;
}
async function defaulted() {
  return withDefault`SELECT 1`;
}
async function aliasedMember() {
  return tag`SELECT 1`;
}
async function letBinding() {
  return fromLet`SELECT 1`;
}
async function varBinding() {
  return doubled`SELECT 1`;
}
async function memberInit() {
  return fromMember`SELECT 1`;
}
async function namedInit() {
  return fromNamed`SELECT 1`;
}
async function otherSource() {
  return fromOther`SELECT 1`;
}
async function otherName() {
  return query`SELECT 1`;
}
async function plainObject() {
  return fromHolder`SELECT 1`;
}

read(missingTag());
read(parameterTag(""));
read(computedMember());
read(templateExpr());
read(computedDestructure());
read(defaulted());
read(aliasedMember());
read(letBinding());
read(varBinding());
read(memberInit());
read(namedInit());
read(otherSource());
read(otherName());
read(plainObject());
